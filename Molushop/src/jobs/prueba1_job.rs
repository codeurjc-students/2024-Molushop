use actix_jobs::Job;

pub struct MyJob;
impl Job for MyJob {
    fn cron(&self) -> &str {
        "*/2 * * * * * *" // every two seconds
    }

    fn run(&mut self) {
        println!("Sending an email to all our clients...");
    }
}