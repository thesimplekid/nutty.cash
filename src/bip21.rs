use crate::types::{AddressParam, AddressParamType};
use url::Url;

pub fn create_bip21(params: &[AddressParam]) -> Result<String, String> {
    if params.is_empty() {
        return Err("No payment options provided".into());
    }

    let base = "bitcoin:".to_string();
    let mut url = Url::parse(&base).map_err(|e| e.to_string())?;

    for param in params {
        match param.kind {
            AddressParamType::LNO => {
                url.query_pairs_mut().append_pair("lno", &param.value);
            }
            AddressParamType::SP => {
                url.query_pairs_mut().append_pair("sp", &param.value);
            }
            AddressParamType::CREQ => {
                url.query_pairs_mut().append_pair("creq", &param.value);
            }
            AddressParamType::CUSTOM => {
                if let Some(ref prefix) = param.prefix {
                    url.query_pairs_mut().append_pair(prefix, &param.value);
                }
            }
        }
    }

    let res = url.to_string();
    if res == "bitcoin:" {
        return Err("No payment options provided".into());
    }

    Ok(res)
}
