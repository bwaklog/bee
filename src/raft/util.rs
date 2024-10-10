use rand::Rng;

#[allow(dead_code)]
pub fn nodeid_rand_u64() -> u32 {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    (num / 100) as u32
}

// #[cfg(test)]
// mod tests {
//     // use super::nodeid_rand_u64;

//     // pretty useless
//     // #[test]
//     // fn test_random() {
//     //     println!("{}", nodeid_rand_u64());
//     // }
// }
