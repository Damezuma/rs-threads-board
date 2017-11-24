use super::rouille;
use super::serde_json;
use super::templates;
use common::{ ResponseContentType};
use user::User;

pub fn signup(_:ResponseContentType)->rouille::Response{
    let mut s = Vec::new();
    templates::signup(&mut s).unwrap();
    return rouille::Response::from_data("text/html;charset=utf-8", s);
}
pub fn signup_ok(ctype: ResponseContentType)->rouille::Response{
    let v = "{msg:\"가입이 완료되었습니다.\"}";
    rouille::Response::text(v).with_additional_header("Content-Type","application/json")
}
pub fn signin_ok(ctype:ResponseContentType, user:&User, token:String)->rouille::Response{
    #[derive(Serialize, Deserialize, Debug)]
    struct SignRes{
        token:String,
        gravatar:String,
        nickname:String
    };
    let s = SignRes{
        token:token,
        gravatar:user.get_gravatar_url(Some(34)),
        nickname:user.get_nickname().unwrap().clone()
    };
    let v = try_or_400!(serde_json::to_vec(&s));
    rouille::Response::from_data("application/json", v)
}