use byteorder::{BigEndian, ByteOrder};
use cuckoo::Cuckoo;
use std::panic;

fn newC(data_bytes_len: usize) -> Cuckoo {
    Cuckoo::new(16, 8 * data_bytes_len, 6) // params:(N,M,cycleLen).  difficulty depends on M/N ratio. See Cuckoo paper
}

pub fn verify_pow(data_base64: &str, pow_proof_base64: &str) -> Result<bool, PowVerifErr> {
    let data_bytes = base64::decode(&data_base64).map_err(PowVerifErr::B64DecodeBody)?;
    let pow_proof_bytes =
        base64::decode(&pow_proof_base64).map_err(PowVerifErr::B64DecodePowProof)?;
    let pow_proof_vec32 = vec8tovec32(&pow_proof_bytes)?;

    let verif_ok = newC(data_bytes.len()).verify(&data_bytes, &pow_proof_vec32);
    Ok(verif_ok)
}

#[cfg(test)] // unwrap is okay for tests
pub fn solve_pow_proof_b64(data_bytes: &[u8]) -> String {
    let pow_proof_vec32 = newC(data_bytes.len()).solve(&data_bytes).unwrap();
    let pow_proof_bytes = vec32tovec8(&pow_proof_vec32);
    let pow_proof_b64 = base64::encode(&pow_proof_bytes);
    pow_proof_b64
}

fn vec8tovec32(vec8: &[u8]) -> Result<Vec<u32>, PowVerifErr> {
    let res_vec32 = panic::catch_unwind(|| {
        let mut vec32: Vec<u32> = vec![0; vec8.as_ref().len() / 4];
        BigEndian::read_u32_into(vec8.as_ref(), &mut vec32);
        vec32
    })
    .map_err(|_| PowVerifErr::Vec8toVec32);

    res_vec32
}
#[cfg(test)] // unwrap is okay for tests
fn vec32tovec8(vec32: &Vec<u32>) -> Vec<u8> {
    let mut vec8: Vec<u8> = Vec::new();
    for elem in vec32 {
        use byteorder::WriteBytesExt;
        vec8.write_u32::<BigEndian>(*elem).unwrap();
    }
    vec8
}

#[derive(thiserror::Error, Debug)]
pub enum PowVerifErr {
    #[error(transparent)]
    B64DecodeBody(base64::DecodeError),
    #[error(transparent)]
    B64DecodePowProof(base64::DecodeError),
    #[error("failed vec<u8> to vec<u32>")]
    Vec8toVec32,
}
