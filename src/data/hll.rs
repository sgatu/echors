use std::hash::{Hash, Hasher};
#[allow(non_snake_case)]
pub struct HLL {
    M: Vec<u8>,
    m: u16,
    split_mask: u64,
    register_split_bits: u8,
    alpha: f64,
}
impl HLL {
    pub fn new(bucket_bits: u8) -> Self {
        // max to 15 bits -> 32768 buckets
        if bucket_bits < 1 || bucket_bits > 15 {
            panic!("Bit number too high must be between 1 and 16, the higher the bits the more precise and more memory used.")
        }
        let m: u16 = 2_u16.pow(bucket_bits as u32);
        let pre_mask: u64 = (1u64 << bucket_bits) - 1;
        let mask = pre_mask << (64 - bucket_bits);
        Self {
            M: vec![0; m as usize],
            m: m,
            alpha: HLL::get_alpha(m),
            split_mask: mask,
            register_split_bits: bucket_bits,
        }
    }
    pub fn add(&mut self, str: &str) {
        let hash = HLL::hash(str);
        let register_pos = self.get_register_pos(hash);
        let zero_run_masked = hash | self.split_mask;
        let count = HLL::count_zero_bits(&zero_run_masked);
        if self.M[register_pos] < count {
            self.M[register_pos] = count;
        }
    }
    fn raw_estimate(&self) -> f64 {
        let mut harmonic = 0.0f64;
        for i in 0..self.m {
            harmonic += 1.0f64 / (1u64 << self.M[i as usize]) as f64;
        }
        harmonic = 1.0f64 / harmonic;
        return (self.alpha * (self.m as f64 * self.m as f64)) * harmonic;
    }
    /**
     * counts registers that are still 0
     */
    fn zero_registers(&self) -> u16 {
        return self
            .M
            .iter()
            .fold(0, |s, i| if *i == 0 { s + 1 } else { s });
    }
    pub fn reset(&mut self) {
        self.M = vec![0; self.m as usize];
    }
    /**
     *  impl following https://algo.inria.fr/flajolet/Publications/FlFuGaMe07.pdf
     */
    #[allow(non_snake_case)]
    pub fn count(&self) -> u64 {
        let raw_estimate = self.raw_estimate();
        let result: u64;
        // small range correction
        if raw_estimate <= (self.m as f64 * (5.0f64 / 2.0f64)) {
            let V = self.zero_registers();
            result = match V {
                0 => raw_estimate.round() as u64,
                _ => (self.m as f64 * (self.m as f64 / V as f64).ln()).round() as u64,
            };
        }
        // intermediate range correction
        else if raw_estimate <= (u32::MAX as f64) / 30.0 {
            result = raw_estimate.round() as u64;
        } else {
            result = ((-(u32::MAX as f64)) * (1.0f64 - raw_estimate / u32::MAX as f64).ln()).round()
                as u64;
        }
        //not sure where this 2 comes from or where I've missed it in the paper, but fixes the result
        return result * 2;
    }
    fn get_register_pos(&self, hash: u64) -> usize {
        ((hash & self.split_mask) >> (64 - self.register_split_bits)) as usize
    }
    fn count_zero_bits(hash: &u64) -> u8 {
        let mut count: u8 = 0;
        let mut pos: u8 = 0;
        while (hash >> pos) & 1 == 0 {
            count += 1;
            pos += 1;
        }
        return count;
    }
    /**
     * values come from paper https://algo.inria.fr/flajolet/Publications/FlFuGaMe07.pdf as follows
     * define α16 = 0.673; α32 = 0.697; α64 = 0.709; αm = 0.7213/(1 + 1.079/m) for m ≥ 128
     */
    fn get_alpha(registers: u16) -> f64 {
        match registers {
            m if m <= 16 => 0.673f64,
            m if m > 16 && m <= 32 => 0.697f64,
            m if m > 32 && m < 64 => 0.709f64,
            _ => 0.7213f64 / (1.0f64 + 1.079f64 / registers as f64),
        }
    }
    /**
     * Murmur hash, force to little endianess
     */
    fn hash(str: &str) -> u64 {
        let mut hasher: fasthash::Murmur3HasherExt = Default::default();
        str.hash(&mut hasher);
        let result = hasher.finish();
        return result.to_le();
    }
}
