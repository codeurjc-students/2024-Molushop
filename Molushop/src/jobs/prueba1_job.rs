use actix_jobs::Job;

pub struct MyJob;
impl Job for MyJob {
    fn cron(&self) -> &str {
        "*/2 * * * * * *" // every two seconds
    }

    fn run(&mut self) {
        tracing::debug!("example job 1 ran");
    }
}