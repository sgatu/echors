use crate::data::HLL;
#[test]
fn hll_test() {
    let mut hll = HLL::new(14);
    for _i in 1..3000 {
        let s: String = uuid::Uuid::new_v4().to_string();
        hll.add(&s);
    }
    println!("count {}", hll.count());
    hll.reset();
}
