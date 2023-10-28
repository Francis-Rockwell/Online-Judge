pub mod api;
pub mod config;
pub mod structs;

use crate::{
    config::{args, config, Config},
    structs::{string2result, string2state, Case, Contest, JobRequest, JobResponse, User},
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use api::{
    contest_id, delete_job, exit, get_contests, get_jobs, get_users, greet, job_id, post_contest,
    post_job, post_user, put_job, ranklist,
};
use env_logger;
use lazy_static::lazy_static;
use mysql::prelude::*;
use mysql::*;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(config(&args()).unwrap()));
    // transform config file to Config struct
    static ref JOB_LIST: Arc<Mutex<Vec::<JobResponse>>> = Arc::new(Mutex::new(vec![]));
    // to store all valid JobResposes
    static ref USER_LIST: Arc<Mutex<Vec::<User>>> = Arc::new(Mutex::new(vec![User {
        id: Some(0),
        name: String::from("root"),
        waring: None
    }]));
    // to store all users, user0 initiated with name "root"
    static ref CONTEST_LIST: Arc<Mutex<Vec::<Contest>>> = Arc::new(Mutex::new(vec![Contest {
        id: Some(0),
        name: String::from(""),
        from: String::from("0001-01-01T02:00:00.001Z"),
        to: String::from("9999-12-31T23:59:59.999Z"),
        problem_ids: vec![],
        user_ids: vec![0],
        submission_limit: 9999,
        waring: None
    }]));
    // to store all contests, contest0 initiated with user0
    static ref MYSQL: Arc<Mutex<Result<Pool>>> = Arc::new(Mutex::new(Pool::new(
        "mysql://Francis:youzirui1017@127.0.0.1:3306/oj"
    )));
    // try connect to mysql
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    {
        let ctlist = &mut CONTEST_LIST.lock().unwrap();
        // contests
        let joblist = &mut JOB_LIST.lock().unwrap();
        // jobs
        let userlist = &mut USER_LIST.lock().unwrap();
        // users
        let config = &CONFIG.lock().unwrap();
        // Config
        let conn = &mut MYSQL.lock().unwrap();
        // mysql
        if conn.is_err() {
            // fail to connect to mysql
            let mut pids = vec![];
            for i in 0..config.problems.len() {
                pids.push(config.problems[i].id);
            }
            ctlist[0].problem_ids = pids;
            //update contest0 with all problem ids
        } else {
            // succeed to connect to mysql
            let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
            if config.flush.unwrap() {
                conn.query_drop(r"TRUNCATE TABLE contest_list").unwrap();
                conn.query_drop(r"TRUNCATE TABLE contest_problems").unwrap();
                conn.query_drop(r"TRUNCATE TABLE contest_users").unwrap();
                conn.query_drop(r"TRUNCATE TABLE job_cases").unwrap();
                conn.query_drop(r"TRUNCATE TABLE job_submit").unwrap();
                conn.query_drop(r"TRUNCATE TABLE joblist").unwrap();
                conn.query_drop(r"TRUNCATE TABLE userlist").unwrap();
            }
            // if '-f', clear data in mysql
            let users: Vec<(usize, String)> = conn.query("SELECT id, name FROM userlist;").unwrap();
            for i in 0..users.len() {
                if users[i].0 == 0 {
                    userlist[0].name = users[i].1.clone();
                    //when there is a user0 in mysql, use its name instead of root in userlist
                } else {
                    userlist.push(User {
                        id: Some(users[i].0),
                        name: users[i].1.clone(),
                        waring: None,
                    });
                }
            }
            // load users from mysql to userlist
            let jobs: Vec<(usize, String, String, String, String, f64)> = conn
                .query("SELECT id, create_time, update_time, state, result, score FROM joblist;")
                .unwrap();
            let submits: Vec<(usize, String, String, usize, usize, usize)> = conn.query(
                "SELECT id, source_code, language, user_id, contest_id, problem_id FROM job_submit;"
            ).unwrap();
            let allcases: Vec<(usize, usize, String, u128, f64, String)> = conn
                .query("SELECT jobid, caseid, result, time, memory, info FROM job_cases;")
                .unwrap();
            for i in 0..jobs.len() {
                let mut cases = vec![];
                for j in 0..allcases.len() {
                    if allcases[j].0 == jobs[i].0 {
                        cases.push(Case {
                            id: allcases[j].1,
                            result: string2result(&allcases[j].2).unwrap(),
                            time: allcases[j].3,
                            memory: allcases[j].4,
                            info: allcases[j].5.clone(),
                        });
                    }
                }
                joblist.push(JobResponse {
                    id: jobs[i].0,
                    created_time: jobs[i].1.clone(),
                    updated_time: jobs[i].2.clone(),
                    submission: JobRequest {
                        source_code: submits[i].1.clone(),
                        language: submits[i].2.clone(),
                        user_id: submits[i].3,
                        contest_id: submits[i].4,
                        problem_id: submits[i].5,
                    },
                    state: string2state(&jobs[i].3).unwrap(),
                    result: string2result(&jobs[i].4).unwrap(),
                    score: jobs[i].5,
                    cases,
                    warning: None,
                });
            }
            // load jobs from mysql to joblist
            let cts: Vec<(usize, String, String, String, usize)> = conn
                .query("SELECT id, name, fromtime, totime, submission_limit FROM contest_list;")
                .unwrap();
            let pids: Vec<(usize, usize)> =
                conn.query("SELECT id, pid FROM contest_problems;").unwrap();
            let uids: Vec<(usize, usize)> =
                conn.query("SELECT id, uid FROM contest_users;").unwrap();
            for i in 0..cts.len() {
                let mut problem_ids = vec![];
                for j in 0..pids.len() {
                    if pids[j].0 == cts[i].0 {
                        problem_ids.push(pids[j].1);
                    }
                }
                let mut user_ids = vec![];
                for j in 0..uids.len() {
                    if uids[j].0 == cts[i].0 {
                        user_ids.push(uids[j].1);
                    }
                }
                if cts[i].0 == 0 {
                    ctlist[0].name = cts[i].1.clone();
                    ctlist[0].from = cts[i].2.clone();
                    ctlist[0].to = cts[i].3.clone();
                    ctlist[0].user_ids = user_ids;
                    ctlist[0].submission_limit = cts[i].4;
                } else {
                    ctlist.push(Contest {
                        id: Some(cts[i].0),
                        name: cts[i].1.clone(),
                        from: cts[i].2.clone(),
                        to: cts[i].3.clone(),
                        problem_ids,
                        user_ids,
                        submission_limit: cts[i].4,
                        waring: None,
                    });
                }
            }
            // load contests from mysql to ctlist
            let stmt = conn
                .prep(
                    "
                INSERT INTO userlist (id, name) 
                SELECT ?, ?  
                from DUAL  
                where not exists(select * from userlist where id = ?)",
                )
                .unwrap();
            conn.exec_iter(stmt, (0, "root", 0)).unwrap();
            // if there is no (0, *) user in mysql, insert (0, root)
            let mut pids = vec![];
            for i in 0..config.problems.len() {
                let stmt = conn
                    .prep(
                        "
                    INSERT INTO contest_problems (id, pid) 
                    SELECT ?, ?  
                    from DUAL  
                    where not exists(select * from contest_problems where id = ? and pid = ?)",
                    )
                    .unwrap();
                conn.exec_iter(stmt, (0, config.problems[i].id, 0, config.problems[i].id))
                    .unwrap();
                pids.push(config.problems[i].id);
            }
            ctlist[0].problem_ids = pids;
            // if there is a problem id not in contest0 in mysql, insert it
            // update contest0 with all problem ids
            let stmt = conn
                .prep(
                    "
                    INSERT INTO contest_users (id, uid) 
                    SELECT ?, ?  
                    from DUAL  
                    where not exists(select * from contest_users where id = ? and uid = ?)",
                )
                .unwrap();
            conn.exec_iter(stmt, (0, 0, 0, 0)).unwrap();
            // if there is no (0, *) user in contest0 in mysql, insert one
            let stmt = conn
                .prep(
                    "
                    INSERT INTO contest_list (id, name, fromtime, totime, submission_limit) 
                    SELECT ?, ?, ?, ?, ?
                    from DUAL  
                    where not exists(select * from contest_list where id = ?)",
                )
                .unwrap();
            conn.exec_iter(
                stmt,
                (
                    0,
                    "",
                    "0001-01-01T02:00:00.001Z",
                    "9999-12-31T23:59:59.999Z",
                    9999,
                    0,
                ),
            )
            .unwrap();
            // if contest0 is not in mysql, insert it
        }
    }
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            .service(post_job)
            .service(get_jobs)
            .service(job_id)
            .service(put_job)
            .service(delete_job)
            .service(post_user)
            .service(get_users)
            .service(post_contest)
            .service(get_contests)
            .service(contest_id)
            .service(ranklist)
            // DO NOT REMOVE: used in automatic testing
            .service(exit)
    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}
