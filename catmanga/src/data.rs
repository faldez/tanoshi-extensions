use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub props: Props,
    pub page: String,
    pub query: Query,
    pub build_id: String,
    pub is_fallback: bool,
    pub gsp: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
    pub page_props: PageProps,
    #[serde(rename = "__N_SSG")]
    pub n_ssg: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
    pub series: Vec<series::Series>,
    pub latests: Vec<Vec<latest::Latest>>,
    pub featured: Vec<featured::Featured>
}

mod series {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Series {
        #[serde(rename = "alt_titles")]
        pub alt_titles: Vec<String>,
        pub authors: Vec<String>,
        pub genres: Vec<String>,
        pub chapters: Vec<Chapter>,
        pub title: String,
        #[serde(rename = "series_id")]
        pub series_id: String,
        pub description: String,
        pub status: String,
        #[serde(rename = "cover_art")]
        pub cover_art: CoverArt,
        #[serde(rename = "all_covers")]
        pub all_covers: Vec<AllCover>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Chapter {
        pub title: Option<String>,
        pub groups: Vec<String>,
        pub number: f64,
        pub volume: Option<i64>,
        #[serde(rename = "display_number")]
        pub display_number: Option<String>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CoverArt {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AllCover {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }
}

mod latest {
    use serde::{Deserialize, Serialize};
    
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Latest {
        #[serde(rename = "alt_titles")]
        #[serde(default)]
        pub alt_titles: Vec<String>,
        #[serde(default)]
        pub authors: Vec<String>,
        #[serde(default)]
        pub genres: Vec<String>,
        #[serde(default)]
        pub chapters: Vec<Chapter>,
        pub title: Option<String>,
        #[serde(rename = "series_id")]
        pub series_id: Option<String>,
        pub description: Option<String>,
        pub status: Option<String>,
        #[serde(rename = "cover_art")]
        pub cover_art: Option<CoverArt>,
        #[serde(rename = "all_covers")]
        #[serde(default)]
        pub all_covers: Vec<AllCover>,
        #[serde(default)]
        pub groups: Vec<String>,
        pub number: Option<i64>,
        pub volume: Option<i64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Chapter {
        pub title: Option<String>,
        pub groups: Vec<String>,
        pub number: f64,
        pub volume: Option<i64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CoverArt {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AllCover {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }
}

mod featured {
    use serde::{Deserialize, Serialize};
    
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Featured {
        #[serde(rename = "alt_titles")]
        pub alt_titles: Vec<String>,
        pub authors: Vec<String>,
        pub genres: Vec<String>,
        pub chapters: Vec<Chapter>,
        pub title: String,
        #[serde(rename = "series_id")]
        pub series_id: String,
        pub status: String,
        pub description: String,
        #[serde(rename = "cover_art")]
        pub cover_art: CoverArt,
        #[serde(rename = "all_covers")]
        pub all_covers: Vec<AllCover>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Chapter {
        pub title: Option<String>,
        pub groups: Vec<String>,
        pub number: f64,
        pub volume: Option<i64>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CoverArt {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AllCover {
        pub source: String,
        pub width: i64,
        pub height: i64,
    }
}
