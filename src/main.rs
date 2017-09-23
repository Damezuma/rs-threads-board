#[macro_use]
extern crate rouille;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
#[macro_use]
extern crate mysql;

use templates::*;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use rouille::Server;
use std::time::Duration;
use std::io;
use std::io::prelude::*;
mod model;
use model::Model;
#[derive(Serialize, Deserialize, Debug)]
struct DbSetting{
	host:String,
	db:String,
	user:String,
	password:String
}
enum ResponseContentType{
    Html,
    Json,
    Xml
}
fn check_accept_type(request:&rouille::Request)->Option<ResponseContentType>{
    let accept:&str = request.header("accept").unwrap_or("text/html");
    let accept_types = accept.split(",");
    let select_accept_type = accept_types.max_by(|one, two|{
        let mut s:Vec< _ > = one.split("q=").collect();
        let v1:i32 = if s.len() == 1{
            10
        }
        else{
            eprintln!("{}",s[1]);
            let t:f32 = s[1].parse().unwrap();
            (t * 10f32) as i32
        };
        s = two.split("q=").collect();
        let v2:i32 = if s.len() == 1{
            10
        }
        else{
            eprintln!("{}",s[1]);
            let t:f32 = s[1].parse().unwrap();
            (t * 10f32) as i32
        };
        match v1.cmp(&v2){
            std::cmp::Ordering::Equal=>std::cmp::Ordering::Greater,
            t@_=>t
        }
    }).unwrap();
    let v:Vec<&str> = select_accept_type.split("/").collect();
    match v[1]{
        "html"|"xhtml"=>Some(ResponseContentType::Html),
        "json"=>Some(ResponseContentType::Json),
        "xml"=>Some(ResponseContentType::Xml),
        _=>None
    }
}
fn main() {
    
    let setting:DbSetting;
    {
        let f =std::fs::File::open("./setting.json").unwrap();
        setting =serde_json::from_reader(f).unwrap();

    } 
    eprintln!("{:?}",setting);
    let mut builder = mysql::OptsBuilder::default();
    builder.ip_or_hostname(Some(setting.host.as_str()))
		.db_name(Some(setting.db.as_str()))
		.user(Some(setting.user.as_str()))
		.pass(Some(setting.password.as_str()))
		.tcp_keepalive_time_ms(Some(1000))
		.read_timeout(Some(Duration::new(2,0)))
		.write_timeout(Some(Duration::new(2,0)))
        .prefer_socket(false)
        .tcp_port(3306);
    let pool = mysql::Pool::new(mysql::Opts::from(builder)).unwrap();

	println!("Now listening on localhost:9999");
	// The `start_server` starts listening forever on the given address.
	let server = Server::new("127.0.0.1:9999", move |request| {
        let mut model = try_or_400!(pool.get_conn());
		router!(request,
            (GET) (/)=>{
                /*
                if let Some(res_type) = check_accept_type(request){
                    let mut s = Vec::new();
                    templates::default(&mut s).unwrap();
                    rouille::Response::from_data("text/html;charset=utf-8", s)
                }
                else{
                    let mut s = Vec::new();
                    templates::default(&mut s).unwrap();
                    rouille::Response::from_data("text/html;charset=utf-8", s)
                }
                */
                let list = model.get_threads_list();
                let mut s = Vec::new();
                templates::default(&mut s).unwrap();
                rouille::Response::from_data("text/html;charset=utf-8", s)
            },
            (GET) (/threads)=>{
                
                rouille::Response::text("스레드 리스트 반환")
            },
            (GET) (/threads/)=>{
                rouille::Response::text("스레드 리스트 반환")
            },
            (POST) (/threads)=>{
                rouille::Response::text("스레드 생성")
            },
            (GET) (/threads/{id:String})=>{
                eprint!("{}",id);
                rouille::Response::text("스레드 뷰")
            },
            (DELETE) (/threads/{id:String})=>{
                eprint!("{}",id);
                rouille::Response::text("스레드 삭제")
            },
            (GET) (/threads/{id:String}/comments)=>{
                eprint!("{}",id);
                rouille::Response::text("스레드 코멘트 로드")
            },
            (POST) (/threads/{id:String}/comments)=>{
                eprint!("{}",id);
                rouille::Response::text("스레드 코멘트 추가")
            },
            (GET) (/threads/{id:String}/comments/{c_id:String})=>{
                eprint!("{}, {}",id, c_id);
                rouille::Response::text("코멘트 정보 뷰")
            },
            (PUT) (/threads/{id:String}/comments/{c_id:String})=>{
                eprint!("{}, {}",id, c_id);
                rouille::Response::text("코멘트 수정")
            },
            (DELETE) (/threads/{id:String}/comments/{c_id:String})=>{
                eprint!("{}, {}",id, c_id);
                rouille::Response::text("코멘트 삭제")
            },
            (POST) (/threads/{id:String}/comments/{c_id:String}/thumbsup)=>{
                eprint!("{}, {}",id, c_id);
                rouille::Response::text("코멘트 추천")
            },
            (POST) (/threads/{id:String}/comments/{c_id:String}/thumbsdown)=>{
                eprint!("{}, {}",id, c_id);
                rouille::Response::text("코멘트 추천")
            },
            (GET) (/tags)=>{
                rouille::Response::text("태그 리스트")
            },
            (GET) (/tags/{tag:String}/threads)=>{
                eprint!("{}",tag);
                rouille::Response::text("태그가 붙여진 스레드 리스트")
            },
            (POST) (/users)=>{
                rouille::Response::text("회원가입")
            },
            (GET) (/users/{user_name:String})=>{
                eprint!("{}",user_name);
                rouille::Response::text("회원 정보")
            },
            (PUT) (/users/{user_name:String})=>{
                eprint!("{}",user_name);
                rouille::Response::text("회원정보 수정")
            },
            (GET) (/login)=>{
                rouille::Response::text("로그인 폼")
            },
			(POST) (/users/{user_name:String}/login)=>{
                eprint!("{}",user_name);
                rouille::Response::text("로그인")
			},
            (POST) (/users/{user_name:String}/logout)=>{
                eprint!("{}",user_name);
                rouille::Response::text("로그아웃")
			},
			
			(GET) (/css/{css:String}) =>{
				let css_path = Path::new("./css").join(css);
				//println!("{:?}",css_path.as_path());
				if let Ok(file) = File::open(css_path){
					rouille::Response::from_file("text/css",file)
				}
				else{
					rouille::Response::empty_404()
				}
			},
			(GET) (/font/{font:String}) =>{
				let font_path = Path::new("./font").join(font);
				//println!("{:?}",font_path.as_path());
				if let Ok(file) = File::open(font_path){
					rouille::Response::from_file("application/font",file)
				}
				else{
					rouille::Response::empty_404()
				}
			},
			(GET) (/js/{js:String}) =>{
				let js_path = Path::new("./js").join(js);
				//println!("{:?}",js_path.as_path());
				if let Ok(file) = File::open(js_path){
					rouille::Response::from_file("script/javascript",file)
				}
				else{
					rouille::Response::empty_404()
				}
			},
			// The code block is called if none of the other blocks matches the request.
			// We return an empty response with a 404 status code.
			_ => rouille::Response::empty_404()
		)
	}).unwrap();
	println!("Listening on {:?}", server.server_addr());
	server.run();
}
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
