use crypto::{
  digest::Digest,
  sha2::Sha256
};

pub fn get_shortend_url(url: &str)->String{
  let mut sha = Sha256::new();
  sha.input_str(url);
  let mut stringify = sha.result_str();
  stringify.truncate(5);
  stringify
}