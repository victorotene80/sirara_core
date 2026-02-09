use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct MonnifyLoginBody {
    pub accessToken: String,
    pub expiresIn: i64, 
}