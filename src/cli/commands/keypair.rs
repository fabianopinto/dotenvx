use crate::crypto::Keypair;
use crate::utils::Result;

pub fn keypair_command(_format: &str) -> Result<()> {
    let keypair = Keypair::generate();

    println!("DOTENV_PUBLIC_KEY=\"{}\"", keypair.public_key());
    println!("DOTENV_PRIVATE_KEY=\"{}\"", keypair.private_key());

    Ok(())
}
