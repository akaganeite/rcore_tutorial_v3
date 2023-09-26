use std::env;
use backtrace::Backtrace;
fn pwd() {
    println!("Hello, world!");
    
    let path=match env::current_dir(){
        Ok(dir)=>dir,
        Err(err)=>panic!("Problem with pwd: {:?}", err),
    };
    println!("{}", path.display());
}

fn dump_stack_test() {
    let bt = Backtrace::new();
    println!("backtrace dump start ===============");
    println!("{:?}", bt);
}

fn main()
{
    pwd();
    dump_stack_test();
    loop{}
}

