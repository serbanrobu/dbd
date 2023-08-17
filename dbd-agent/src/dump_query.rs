use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Default)]
pub struct DumpQuery {
    #[serde(deserialize_with = "deserialize", default)]
    pub exclude_table_data: Option<Vec<String>>,
}

fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .map(|s| s.split(',').map(str::to_owned).collect()))
}
