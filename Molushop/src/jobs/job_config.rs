use actix_jobs::{Scheduler, run_forever};

use super::prueba1_job::MyJob;
use super::prueba2_job::MyJob2;

//importar todos los struct de jobs
pub fn init_jobs() {
    let mut scheduler = Scheduler::new();
    scheduler.add(Box::new(MyJob));
    scheduler.add(Box::new(MyJob2));
    run_forever(scheduler);
} 