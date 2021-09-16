use nepnep::tanoshi_lib::prelude::*;
use std::str::FromStr;

const ID: i64 = 3;
const NAME: &str = "mangasee";
const URL: &str = "https://mangasee123.com";
const ICON_URL: &str = "https://mangasee123.com/media/favicon.png";

#[derive(Default)]
pub struct Mangasee;

nepnep::generate!(Mangasee);
