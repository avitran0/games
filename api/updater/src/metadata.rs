use std::path::Path;

use quick_xml::{
    Reader, Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

const MANAGED_FIELDS: &[&str] = &[
    "name",
    "desc",
    "image",
    "marquee",
    "thumbnail",
    "video",
    "releasedate",
    "developer",
    "publisher",
    "genre",
    "players",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Element(Element),
    Text(String),
}

pub(super) fn merge(existing: Option<&str>, incoming: &str) -> Result<Vec<u8>, String> {
    let incoming_root = parse(incoming)?;
    if incoming_root.name != "gameList" {
        return Err("Incoming metadata must use <gameList> as its root element".into());
    }

    let mut target_root = match existing {
        Some(existing) => parse(existing)?,
        None => Element::new("gameList"),
    };
    if target_root.name != "gameList" {
        return Err("The existing metadata must use <gameList> as its root element".into());
    }

    let mut seen = Vec::new();
    for incoming_game in incoming_root.children_named("game") {
        let path = incoming_game
            .child_text("path")
            .ok_or_else(|| "Metadata entry has no launcher path".to_owned())?;
        validate_path(&path)?;
        if seen.iter().any(|previous| previous == &path) {
            return Err(format!(
                "The metadata has the same launcher path more than once: {path}."
            ));
        }
        seen.push(path.clone());

        let target_game = target_root
            .children_named_mut("game")
            .find(|game| game.child_text("path").as_deref() == Some(&path));
        let game = match target_game {
            Some(game) => game,
            None => {
                let mut game = Element::new("game");
                game.children
                    .push(Node::Element(Element::with_text("path", &path)));
                target_root.children.push(Node::Element(game));
                target_root
                    .children_named_mut("game")
                    .last()
                    .expect("the new game was appended")
            }
        };

        for field in MANAGED_FIELDS {
            let Some(value) = incoming_game.child(field) else {
                continue;
            };
            let value = value.text_content();
            if let Some(current) = game.child_mut(field) {
                current.children.clear();
                current.children.push(Node::Text(value));
            } else {
                game.children
                    .push(Node::Element(Element::with_text(*field, &value)));
            }
        }
    }

    write(&target_root)
}

fn validate_path(path: &str) -> Result<(), String> {
    let Some(relative) = path.strip_prefix("./") else {
        return Err(format!("The launcher path is invalid: {path:?}"));
    };
    if relative.is_empty()
        || Path::new(relative).is_absolute()
        || relative
            .split(['/', '\\'])
            .any(|part| part == ".." || part.is_empty())
    {
        return Err(format!("The launcher path is unsafe: {path:?}"));
    }
    Ok(())
}

fn parse(input: &str) -> Result<Element, String> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<Element> = Vec::new();
    let mut root = None;

    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Start(start) => stack.push(read_element(&start)?),
            Event::Empty(start) => append_element(&mut stack, &mut root, read_element(&start)?)?,
            Event::End(_) => {
                let element = stack.pop().ok_or_else(|| {
                    "The metadata file has a closing tag without an opening tag".to_owned()
                })?;
                append_element(&mut stack, &mut root, element)?;
            }
            Event::Text(text) => {
                let text = text.xml10_content();
                if !text.trim().is_empty() {
                    stack
                        .last_mut()
                        .ok_or_else(|| {
                            "The metadata file has text before or after its root element".to_owned()
                        })?
                        .children
                        .push(Node::Text(text.into_owned()));
                }
            }
            Event::CData(text) => {
                let text = text.xml10_content();
                stack
                    .last_mut()
                    .ok_or_else(|| {
                        "The metadata file has CDATA before or after its root element".to_owned()
                    })?
                    .children
                    .push(Node::Text(text.into_owned()));
            }
            Event::GeneralRef(reference) => {
                let reference = format!("&{};", reference.as_ref());
                let value = quick_xml::escape::unescape(&reference)
                    .map_err(|error| error.to_string())?
                    .into_owned();
                stack
                    .last_mut()
                    .ok_or_else(|| {
                        "The metadata file has an entity reference outside its root element"
                            .to_owned()
                    })?
                    .children
                    .push(Node::Text(value));
            }
            Event::DocType(_) => {
                return Err(
                    "The metadata file uses a DOCTYPE declaration, which is not supported".into(),
                );
            }
            Event::Eof => break,
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
        }
    }

    if !stack.is_empty() {
        return Err("The metadata file has an unclosed element".into());
    }
    root.ok_or_else(|| "The metadata file has no root element".into())
}

fn read_element(start: &BytesStart<'_>) -> Result<Element, String> {
    let name = start.name().as_ref().to_owned();
    let mut element = Element::new(name);
    for attribute in start.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| error.to_string())?;
        let key = attribute.key.as_ref().to_owned();
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Explicit1_0)
            .map_err(|error| error.to_string())?
            .into_owned();
        element.attributes.push((key, value));
    }
    Ok(element)
}

fn append_element(
    stack: &mut [Element],
    root: &mut Option<Element>,
    element: Element,
) -> Result<(), String> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Node::Element(element));
    } else if root.replace(element).is_some() {
        return Err("The metadata file has more than one root element".into());
    }
    Ok(())
}

fn write(root: &Element) -> Result<Vec<u8>, String> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 4);
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|error| error.to_string())?;
    write_element(&mut writer, root)?;
    Ok(writer.into_inner())
}

fn write_element(writer: &mut Writer<Vec<u8>>, element: &Element) -> Result<(), String> {
    let mut start = BytesStart::new(element.name.as_str());
    for (name, value) in &element.attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
    if element.children.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    writer
        .write_event(Event::Start(start))
        .map_err(|error| error.to_string())?;
    for child in &element.children {
        match child {
            Node::Element(child) => write_element(writer, child)?,
            Node::Text(text) => writer
                .write_event(Event::Text(BytesText::new(text)))
                .map_err(|error| error.to_string())?,
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new(element.name.as_str())))
        .map_err(|error| error.to_string())?;
    Ok(())
}

impl Element {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    fn with_text(name: impl Into<String>, text: &str) -> Self {
        let mut element = Self::new(name);
        element.children.push(Node::Text(text.to_owned()));
        element
    }

    fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find_map(|child| match child {
            Node::Element(element) if element.name == name => Some(element),
            _ => None,
        })
    }

    fn child_mut(&mut self, name: &str) -> Option<&mut Element> {
        self.children.iter_mut().find_map(|child| match child {
            Node::Element(element) if element.name == name => Some(element),
            _ => None,
        })
    }

    fn children_named(&self, name: &str) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(move |child| match child {
            Node::Element(element) if element.name == name => Some(element),
            _ => None,
        })
    }

    fn children_named_mut(&mut self, name: &str) -> impl Iterator<Item = &mut Element> {
        self.children
            .iter_mut()
            .filter_map(move |child| match child {
                Node::Element(element) if element.name == name => Some(element),
                _ => None,
            })
    }

    fn child_text(&self, name: &str) -> Option<String> {
        self.child(name).map(Element::text_content)
    }

    fn text_content(&self) -> String {
        self.children
            .iter()
            .map(|child| match child {
                Node::Element(element) => element.text_content(),
                Node::Text(text) => text.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::merge;

    #[test]
    fn updates_managed_fields_and_preserves_other_fields() {
        let existing = r#"<gameList><game><path>./Snake.sh</path><name>Old</name><favorite>true</favorite><players>2</players></game></gameList>"#;
        let incoming = r#"<gameList><game><path>./Snake.sh</path><name>Snake</name><players>1</players><desc>A &amp; B</desc></game></gameList>"#;
        let result = String::from_utf8(merge(Some(existing), incoming).unwrap()).unwrap();
        assert!(result.contains("<name>Snake</name>"));
        assert!(result.contains("<favorite>true</favorite>"));
        assert!(result.contains("<players>1</players>"));
        assert!(result.contains("<desc>A &amp; B</desc>"));
    }

    #[test]
    fn rejects_unsafe_launcher_paths() {
        let incoming = r#"<gameList><game><path>./../outside.sh</path></game></gameList>"#;
        assert!(
            merge(None, incoming)
                .unwrap_err()
                .contains("launcher path is unsafe")
        );
    }

    #[test]
    fn rejects_a_wrong_root() {
        assert!(merge(None, "<root/>").unwrap_err().contains("root"));
    }
}
