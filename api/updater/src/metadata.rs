use std::{collections::HashSet, io::Cursor, path::Path};

use api::serde::Deserialize;
use xmltree::{Element, EmitterConfig, XMLNode};

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

#[derive(Deserialize)]
#[serde(crate = "api::serde")]
struct MetadataFile {
    games: Vec<GameMetadata>,
}

#[derive(Deserialize)]
#[serde(crate = "api::serde")]
struct GameMetadata {
    path: String,
    #[serde(flatten)]
    fields: std::collections::HashMap<String, String>,
}

pub(super) fn merge(existing: Option<&str>, incoming: &str) -> Result<Vec<u8>, String> {
    let incoming: MetadataFile = serde_json::from_str(incoming)
        .map_err(|error| format!("Failed to read game metadata: {error}"))?;
    let mut root = match existing {
        Some(xml) => Element::parse(Cursor::new(xml))
            .map_err(|error| format!("Failed to read the existing game list: {error}"))?,
        None => Element::new("gameList"),
    };
    if root.name != "gameList" {
        return Err("The game list must use <gameList> as its root element.".into());
    }

    let mut seen = HashSet::new();
    for metadata in incoming.games {
        validate_path(&metadata.path)?;
        if !seen.insert(metadata.path.clone()) {
            return Err(format!(
                "The metadata repeats a launcher path: {}.",
                metadata.path
            ));
        }

        let index = root.children.iter().position(|node| {
            node.as_element().is_some_and(|game| {
                game.name == "game" && child_text(game, "path").as_deref() == Some(&metadata.path)
            })
        });
        let game = match index {
            Some(index) => root.children[index]
                .as_mut_element()
                .expect("the selected node is an element"),
            None => {
                let mut game = Element::new("game");
                game.children
                    .push(XMLNode::Element(text_element("path", &metadata.path)));
                root.children.push(XMLNode::Element(game));
                root.children
                    .last_mut()
                    .and_then(XMLNode::as_mut_element)
                    .expect("the new game was appended")
            }
        };

        for field in MANAGED_FIELDS {
            let Some(value) = metadata.fields.get(*field) else {
                continue;
            };
            if let Some(current) = game.get_mut_child(*field) {
                current.children.clear();
                current.children.push(XMLNode::Text(value.clone()));
            } else {
                game.children
                    .push(XMLNode::Element(text_element(field, value)));
            }
        }
    }

    let mut output = Vec::new();
    root.write_with_config(
        &mut output,
        EmitterConfig::new()
            .perform_indent(true)
            .indent_string("    "),
    )
    .map_err(|error| format!("Failed to write the game list: {error}"))?;
    Ok(output)
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

fn child_text(element: &Element, name: &str) -> Option<String> {
    element
        .get_child(name)?
        .get_text()
        .map(|text| text.into_owned())
}

fn text_element(name: &str, text: &str) -> Element {
    let mut element = Element::new(name);
    element.children.push(XMLNode::Text(text.to_owned()));
    element
}

#[cfg(test)]
mod tests {
    use super::merge;

    #[test]
    fn updates_managed_fields_and_preserves_other_fields() {
        let existing = r#"<gameList><game><path>./Snake.sh</path><name>Old</name><favorite>true</favorite><players>2</players></game></gameList>"#;
        let incoming =
            r#"{"games":[{"path":"./Snake.sh","name":"Snake","players":"1","desc":"A & B"}]}"#;
        let result = String::from_utf8(merge(Some(existing), incoming).unwrap()).unwrap();
        assert!(result.contains("<name>Snake</name>"));
        assert!(result.contains("<favorite>true</favorite>"));
        assert!(result.contains("<players>1</players>"));
        assert!(result.contains("<desc>A &amp; B</desc>"));
    }

    #[test]
    fn rejects_unsafe_launcher_paths() {
        let incoming = r#"{"games":[{"path":"./../outside.sh"}]}"#;
        assert!(
            merge(None, incoming)
                .unwrap_err()
                .contains("launcher path is unsafe")
        );
    }

    #[test]
    fn rejects_a_wrong_root() {
        assert!(
            merge(Some("<root/>"), r#"{"games":[]}"#)
                .unwrap_err()
                .contains("root element")
        );
    }
}
