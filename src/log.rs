use colored::*;

pub fn request(input: String) {
    println!("{} {input}", " REQUEST ".on_cyan());
}

pub fn success(input: String) {
    println!("{} {input}", " SUCCESS ".on_green());
}

pub fn failure(input: String) {
    println!("{} {input}", " FAILURE ".on_red());
}