use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Atlas {
    #[serde(rename = "t")]
    pub textures: Vec<Texture>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Texture {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "imgs")]
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    #[serde(rename = "n")]
    pub name: String,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "w")]
    pub width: i32,
    #[serde(rename = "h")]
    pub height: i32,

    #[serde(rename = "fx")]
    pub frame_x: i32,
    #[serde(rename = "fy")]
    pub frame_y: i32,
    #[serde(rename = "fw")]
    pub frame_width: i32,
    #[serde(rename = "fh")]
    pub frame_height: i32,

    #[serde(rename = "r")]
    pub rotated: bool,
}

impl Atlas {
    pub fn write_to_xml_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut file = std::fs::File::create(path)?;

        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut file);
        writer.write(xml::writer::XmlEvent::start_element("Atlas"))?;

        for texture in self.textures.iter() {
            writer
                .write(xml::writer::XmlEvent::start_element("Texture").attr("n", &texture.name))?;

            for image in texture.images.iter() {
                writer.write(
                    xml::writer::XmlEvent::start_element("Image")
                        .attr("n", &image.name)
                        .attr("x", &format!("{}", image.x))
                        .attr("y", &format!("{}", image.y))
                        .attr("w", &format!("{}", image.width))
                        .attr("h", &format!("{}", image.height))
                        .attr("fx", &format!("{}", image.frame_x))
                        .attr("fy", &format!("{}", image.frame_y))
                        .attr("fw", &format!("{}", image.frame_width))
                        .attr("fh", &format!("{}", image.frame_height))
                        .attr("r", if image.rotated { "1" } else { "0" }),
                )?;
                writer.write(xml::writer::XmlEvent::end_element())?;
            }

            writer.write(xml::writer::XmlEvent::end_element())?;
        }

        writer.write(xml::writer::XmlEvent::end_element())?;

        Ok(())
    }
}
