pub struct Rule {
    from: String,
    to: String,
}

pub type Env = Map<&str, [f32; 3]>;

pub fn rule(from: &str, to: &str) -> Rule {
    Rule {
        from: String::from(from),
        to: String::from(to),
    }
}

pub fn run(rules: &[Rule], seed: &str) -> String {
    let mut value = String::from(seed);

    'main: loop {
        for r in rules {
            if value.contains(&r.from) {
                value = value.replacen(&r.from, &r.to, 1);
                continue 'main;
            }
        }
        break value;
    }
}