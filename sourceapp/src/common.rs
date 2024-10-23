use std::{fs, io::{self, Write}, path::Path};

use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
struct SourceData {
    student_name: String,
    classes: Vec<Class>,
    past_classes: Vec<PastClass>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Class {
    assignments: Value,
    assignments_parsed: Vec<Assignment>,
    frn: String,
    store_code: String,
    url: String,
    name: String,
    teacher_name: String,
    teacher_contact: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Assignment {
    _assignmentsections: Vec<AssignmentSection>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AssignmentSection {
    _id: i32,
    _name: String,
    assignmentsectionid: i32,
    duedate: String,
    iscountedinfinalgrade: bool,
    isscorespublish: bool,
    isscoringneeded: bool,
    name: String,
    scoreentrypoints: f32,
    scoretype: String,
    sectionsdcid: i32,
    totalpointvalue: f32,
    weight: f32,
    _assignmentscores: Vec<AssignmentScore>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AssignmentScore {
    _name: String,
    actualscoreentered: Option<String>,
    actualscorekind: Option<String>,
    authoredbyuc: bool,
    isabsent: bool,
    iscollected: bool,
    isexempt: bool,
    isincomplete: bool,
    islate: bool,
    ismissing: bool,
    scoreentrydate: String,
    scorelettergrade: Option<String>,
    scorepercent: Option<f32>,
    scorepoints: Option<f32>,
    studentsdcid: i32,
    whenmodified: String,
}

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Http Error {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Regex Error {0}")]
    RegexError(#[from] regex::Error),
    #[error("Url Parse Error {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Json Parse Error")]
    JsonParseError(#[from] serde_json::Error),
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("Detected Login Failure")]
    LoginFailureError,
    #[error("DOM parse error {0}")]
    DOMParseError(#[from] tl::ParseError)
}

pub fn get_source_data(username: &str, password: &str, download_path: &str) -> Result<String, SourceError> {
    let client = ClientBuilder::new().cookie_store(true).build()?;
    let session_res = client.get("https://ps.seattleschools.org").send()?;
    let login_body  = [
        ("dbpw", password), 
        ("serviceName", "PS Parent Portal"),
        ("pcasServerUrl", "/"),
        ("credentialType", "User Id and Password Credential"),
        ("ldappassword", password),
        ("account", username),
        ("pw", password),
    ];
    session_res.error_for_status()?;
    let login_res = client.post("https://ps.seattleschools.org/guardian/home.html").form(&login_body).send()?.error_for_status()?;
    let home_html = login_res.text()?;
    // fs::File::create("home.html").unwrap().write(home_html.as_bytes()).unwrap();
    get_img_url(&client, download_path)?;

    let scores_html_regex = Regex::new("<a href=\"scores.html\\?frn=[^\\\"]*\"")?;
    let name_regex = Regex::new("<span id=\"firstlast\">[^<]*")?;
    let student_name = name_regex.find(&home_html).map(|name_match| name_match.as_str()["<span id=\"firstlast\">".len()..].to_string());
    let student_name = student_name.ok_or(SourceError::LoginFailureError)?;
    let score_url_regex = Regex::new("\"s[^\"]*")?;
    let class_header_regex = Regex::new("<td class=\"table-element-text-align-start\">[^&]*")?;
    let class_headers = class_header_regex.find_iter(&home_html).map(|class_header_match| {
        class_header_match.as_str()["<td class=\"table-element-text-align-start\">".len()..].to_string()
    }).collect::<Vec<_>>();

    let teachers = get_teachers(&client, &home_html)?;

    let mut class_frns = Vec::new();
    for score_href in scores_html_regex.find_iter(&home_html) {
        let Some(score_url_match) = score_url_regex.find(score_href.as_str()) else {
            println!("couldn't find url from href {}", score_href.as_str());
            continue; 
        };
        let score_url = &score_url_match.as_str()[1..];
        let full_score_url = Url::parse(&format!("https://ps.seattleschools.org/guardian/{}", score_url))?;
        let Some((_, class_frn)) = full_score_url.query_pairs().into_iter().find(|(key, _)| key == "frn") else { continue; };
        let Some((_, store_code)) = full_score_url.query_pairs().into_iter().find(|(key, _)| key == "fg") else { continue; };
        if store_code != "S1" && store_code != "S2" {
            continue;
        }
        class_frns.push((full_score_url.to_string(), class_frn.to_string(), store_code.to_string()));
        
    }
    
    let data_ng_regex: Regex = Regex::new("data-ng-init=\"[^\"]*")?;
    let student_frn_regex: Regex = Regex::new("studentFRN = '[^']*")?;
    let section_id_regex: Regex = Regex::new("data-sectionid=\"[^\"]*")?;

    let mut assignments: Vec<Class> = Vec::new();

    for (i, (full_score_url, class_frn, store_code)) in class_frns.into_iter().enumerate() {
        let scores_html_res = client.get(&full_score_url).send()?;
        let scores_text = scores_html_res.text()?;
        let Some(data_ng) = data_ng_regex.find(&scores_text) else {
            println!("no data_ng found at {full_score_url}");
            continue;
        };

        let Some(section_id_match) = section_id_regex.find(&scores_text) else { continue; };
        let section_id = &section_id_match.as_str()["data-sectionid=\"".len()..];
        let Some(student_frn_match) = student_frn_regex.find(data_ng.as_str()) else { continue; };
        let student_frn = &student_frn_match.as_str()[student_frn_match.as_str().len()-6..];
        println!("section_id: {section_id}, student_frn: {student_frn}, store_code: {store_code}");
        let assignments_res = client.post(format!("https://ps.seattleschools.org/ws/xte/assignment/lookup"))
            .body(format!("{{\"section_ids\":[{section_id}],\"student_ids\":[{student_frn}], \"store_codes\": [\"{store_code}\"]}}"))
            .header("Content-Type", "application/json;charset=UTF-8")
            .header("Referer", &full_score_url)
            .header("Accept", "application/json, text/plain, */*")
            .send()?;
        let assignments_json = assignments_res.text()?;
        assignments.push(Class {
            frn: class_frn.into(),
            assignments: serde_json::from_str(&assignments_json)?, 
            assignments_parsed: serde_json::from_str(&assignments_json)?, 
            url: full_score_url, 
            store_code, 
            name: class_headers[i].clone(),
            teacher_name: teachers[i].0.clone(),
            teacher_contact: teachers[i].1.clone(),
        });
    }
    let past_classes = get_past_grades(&client)?;
    let source_data = SourceData {
        classes: assignments,
        student_name,
        past_classes,
    };
    Ok(serde_json::to_string_pretty(&source_data)?)
}

fn get_img_url(client: &Client, download_path: &str) -> Result<(), SourceError> {
    let photo_html = client.get("https://ps.seattleschools.org/guardian/student_photo.html").send()?.text()?;
    let photo_dom = tl::parse(&photo_html, tl::ParserOptions::default())?;
    let parser = photo_dom.parser();
    if let Some(photo_container_handle) = photo_dom.get_elements_by_class_name("user-photo").next() {
        if let Some(photo_container) = photo_container_handle.get(parser) {
            if let Some(children) = photo_container.children() {
                if let Some(image_node) = children.all(parser).first() {
                    if let Some(image_tag) = image_node.as_tag() {
                        if let Some(Some(src)) = image_tag.attributes().get("src") {
                            if let Ok(src_string) = String::from_utf8(src.as_bytes().into()) {
                                let pfp_bytes = client.get(format!("https://ps.seattleschools.org{}", src_string)).send()?.bytes()?;
                                if !Path::new(download_path).exists() {
                                    fs::create_dir(download_path)?;
                                }
                                fs::File::create(Path::new(download_path).join("pfp.jpeg"))?.write(&pfp_bytes)?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct PastClass {
    date_completed: String,
    grade_level: String,
    school: String,
    course_id: String,
    course_name: String,
    credit_earned: f32,
    credit_attempted: f32,
    grade: String,
}

fn get_past_grades(client: &Client) -> Result<Vec<PastClass>, SourceError> {
    let grades_html = client.get("https://ps.seattleschools.org/guardian/termgrades.html").send()?.text()?;
    let grades_dom = Html::parse_document(&grades_html);
    let table_body_selector = Selector::parse("tbody").unwrap();
    if let Some(table_body) = grades_dom.select(&table_body_selector).next() {
        let mut classes = vec![];
        for table_row in table_body.child_elements() {
            if table_row.value().name() == "tr" {
                let table_data = table_row.child_elements().map(|child| {
                    child.inner_html()
                }).collect::<Vec<String>>();
                if table_data.len() == 8 {
                    if let Ok(credit_earned) = table_data[5].trim().parse() {
                        if let Ok(credit_attempted) = table_data[6].trim().parse() {
                            classes.push(PastClass {
                                date_completed: table_data[0].clone(),
                                grade_level: table_data[1].clone(),
                                school: table_data[2].clone(),
                                course_id: table_data[3].clone(),
                                course_name: table_data[4].clone(),
                                credit_earned,
                                credit_attempted,
                                grade: table_data[7].clone(),
                            });
                        } else {
                            println!("attempted is {}", table_data[6].trim());
                        }
                    } else {
                        println!("earned is {}", table_data[5].trim());
                    }
                } else {
                    println!("table data is not 8 it is {}", table_data.len());
                }
            } else {
                println!("found child but not tr {}", table_row.value().name());
            }
        }
        return Ok(classes);
    } else {
        println!("no tbody");
    }
    Ok(vec![])
}

fn get_teachers(client: &Client, home_html: &str) -> Result<Vec<(String, String)>, SourceError>{
    let mut teachers: Vec<(String, String)> = vec![];
    let teacher_url_regex = Regex::new("<a href=\"teacherinfo.html\\?frn=[^\"]*\"")?;
    for teacher_url_match in teacher_url_regex.find_iter(&home_html) {
        let teacher_url = &teacher_url_match.as_str()["<a href=\"".len()..teacher_url_match.len()-1];
        let full_url = format!("https://ps.seattleschools.org/guardian/{teacher_url}");
        let res = client.get(full_url).send()?;
        let teacher_html = res.text()?;
        let teacher_name_regex = Regex::new("<p><strong>Name:</strong>[^<]*")?;
        let Some(teacher_name_match) = teacher_name_regex.find(&teacher_html) else {
            teachers.push(("".into(), "".into()));
            continue;
        };
        let teacher_name = (&teacher_name_match.as_str()["<p><strong>Name:</strong>".len()..]).trim();
        let teacher_contact_regex = Regex::new("<a href=\"[^\"]*")?;
        let Some(teacher_contact_match) = teacher_contact_regex.find(&teacher_html) else {
            teachers.push((teacher_name.into(), "".into()));
            continue;
        };
        let teacher_contact = &teacher_contact_match.as_str()["<a href=\"".len()..];
        teachers.push((teacher_name.into(), teacher_contact.into()));
    }
    Ok(teachers)
}