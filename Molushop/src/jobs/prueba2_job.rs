use actix_jobs::Job;

pub struct MyJob2;
impl Job for MyJob2 {
    fn cron(&self) -> &str {
        "*/5 * * * * * *" // every two seconds
    }

    fn run(&mut self) {
        println!("Otro job");
    }
}