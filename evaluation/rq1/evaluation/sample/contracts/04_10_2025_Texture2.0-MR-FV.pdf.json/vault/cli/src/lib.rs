use solana_sdk::{pubkey::Pubkey, signature::Signature};

pub mod app;

#[serde_with::serde_as]
#[derive(serde::Serialize, display_json::DisplayAsJsonPretty)]
pub struct ResultView<T> {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub signature: Signature,
    #[serde(flatten)]
    pub payload: T,
}

fn created_vault(signature: Signature, vault: Pubkey) -> ResultView<CreatedVault> {
    ResultView {
        signature,
        payload: CreatedVault { vault },
    }
}

#[derive(serde::Serialize)]
pub struct CreatedVault {
    vault: Pubkey,
}

impl From<Signature> for ResultView<()> {
    fn from(signature: Signature) -> Self {
        ResultView {
            signature,
            payload: (),
        }
    }
}

pub fn to_zero_padded<const SIZE: usize>(text: &str) -> anyhow::Result<[u8; SIZE]> {
    let mut array = [0; SIZE];
    let bytes = text.as_bytes();
    array[..bytes.len()].copy_from_slice(bytes);
    Ok(array)
}
