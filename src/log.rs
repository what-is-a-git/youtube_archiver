use colored::*;

pub fn request(input: String) {
    println!("{} {input}", " REQUEST ".black().on_cyan());
}

pub fn success(input: String) {
    println!("{} {input}", " SUCCESS ".black().on_green());
}

pub fn failure(input: String) {
    println!("{} {input}", " FAILURE ".black().on_red());
}
