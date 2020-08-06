#[derive(Debug)]
struct Factoriadic(Vec<u64>);

impl Factoriadic {
    fn new(mut n: u64) -> Factoriadic {
        let mut out = vec![];
        let mut factor = 1;

        loop {
            let rem = n % factor;
            out.push(rem);
            n /= factor;
            factor += 1;

            if n == 0 {
                break;
            }
        }

        Factoriadic(out)
    }

    fn to_rev_vec(&self) -> Vec<u64> {
        self.0.iter().copied().rev().collect()
    }
}

impl std::ops::Add for Factoriadic {
    type Output = Self;

    //works only for factoriadics of equal size
    fn add(self, other: Self::Output) -> Self::Output {
        let mut carry = 0;
        let mut factor = 1;
        let mut out = vec![];

        for (this, other) in self.0.iter().zip(other.0.iter()) {
            let sum = this + other + carry;
            let res = sum % factor;
            carry = (sum / factor != 0) as u64;

            out.push(res);
            factor += 1;
        }

        if carry != 0 {
            out.push(carry);
        }

        Factoriadic(out)
    }
}

impl std::fmt::Display for Factoriadic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fact{:?}", self.to_rev_vec())
    }
}

fn get_ith(from: &[u64], i: u64) -> Vec<u64> {
    let mut from: Vec<_> = from.iter().copied().collect();

    let mut f: Vec<_> = Factoriadic::new(i).to_rev_vec();
    let s = f.len();
    f.resize_with(from.len(), Default::default);
    f.rotate_right(from.len() - s);
    //println!("rev: {:?}", f);

    let mut out = vec![];

    for index in f {
        out.push(from[index as usize]);
        from.remove(index as usize);
    }

    //out.reverse();
    out
}

fn main() {
    for i in 0..29 {
        println!("{} -> {}", i, Factoriadic::new(i));
    }

    println!();

    for i in 0..15 {
        println!("{}*2 -> {}", i, Factoriadic::new(i) + Factoriadic::new(i));
    }

    for i in 0..6 {
        println!("\n{} {:?}", i, get_ith(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], i));
    }

    println!("\n{} {:?}", 3093889, get_ith(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 3093889));
}
