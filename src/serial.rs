use serde::{Serialize, Deserialize};

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
    pub images: Vec<Image>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
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