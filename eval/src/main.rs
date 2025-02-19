use ruler::enumo::Sexp;

fn main() {
    let sexp = Sexp::Atom("hi".into());
    println!("Hello, world!: {}", sexp);
}
