extern crate mysql;
use mysql::prelude::*;
use std;
use model::*;
use user::*;
/*
impl Model for mysql::PooledConn {
    fn get_threads_list(&mut self,offset:usize, count:usize)->Vec<Thread>{
        let sql =format!("SELECT * FROM v_thread_list LIMIT {}, {}", offset, count);
        let params:&[&ToValue] = &[];
        return self.prep_exec(sql,params).unwrap().map(|row|{
            let mut row = row.unwrap();
            Thread::new(
                row.take("uid").expect("uid"),
                row.take("subject").expect("uid"),
                row.take("recent_update").expect("recent_update"),
                row.take("created_datetime").expect("created_datetime"),
                User::new()
                .uid(row.take("opener_uid").expect("opener_uid"))
                .nickname(row.take("opener_nickname").expect("opener_nickname"))
                .email(row.take("opener_email").expect("opener_email"))
            )
        }).collect();
    }
    
    fn get_thread(&mut self, thread_uid:i32)->Option<Thread>{
        let sql =format!("SELECT * FROM v_thread_list WHERE uid = ? LIMIT 1");
        let params:&[&ToValue] = &[&thread_uid];
        let mut thread = match self.first_exec(sql, params).unwrap(){
            Some(v)=>v,
            None=>return None
        };

        
        let res = Thread::new(
            thread.take("uid").expect("uid"),
            thread.take("subject").expect("uid"),
            thread.take("recent_update").expect("recent_update"),
            thread.take("created_datetime").expect("created_datetime"),
            User::new()
                .uid(thread.take("opener_uid").expect("opener_uid"))
                .nickname(thread.take("opener_nickname").expect("opener_nickname"))
                .email(thread.take("opener_email").expect("opener_email"))
        );
        return Some(res);
    }
    // add code here
}
*/
impl Tag{
    pub fn list(model:&mut mysql::PooledConn, q:Option<&str>)->Vec<Tag>{
        if let Some(v) = q{
            let sql = "SELECT * FROM v_tag_threads_count_list where tag_name LIKE ?";
            let params:&[&ToValue] = &[&format!("%{}%", v)];
            //let thread_uids:Vec<i32> = 
            model.prep_exec(sql,params).unwrap().map(|row|{
                let row = row.unwrap();
                return Tag::new(row.get(0).unwrap()).with_thread_count(row.get(1).unwrap());
            }).collect()
        }
        else{

            let sql = "SELECT * FROM v_tag_threads_count_list";
            let params:&[&ToValue] = &[];
            model.prep_exec(sql,params).unwrap().map(|row|{
                let row = row.unwrap();
                return Tag::new(row.get(0).unwrap()).with_thread_count(row.get(1).unwrap());
            }).collect()
        }
        
    }
    pub fn put(&self,model:&mut mysql::PooledConn, thread:&Thread){
        let sql = "INSERT INTO tb_tags VALUES (?, ?)";
        let param:&[&ToValue] = &[self.get_name(), &thread.get_uid()];
        model.prep_exec(sql, param).unwrap();
    }
    pub fn delete(&self,model:&mut mysql::PooledConn, thread:&Thread){
        let sql = "DELETE FROM tb_tags WHERE thread_uid = ?";
        let param:&[&ToValue] = &[&thread.get_uid()];
        model.prep_exec(sql, param).unwrap();
    }
}
impl Thread{
    pub fn e_tag(conn:&mut mysql::PooledConn, offset:usize, q:&Option<String>)->String{
        let row  =if let &Some(ref q) = q{
            let t = format!("%{}%", q);
            let params:&[&ToValue]= &[&t, &offset];
            conn.first_exec("SELECT md5(recent_update) FROM v_thread_list WHERE subject LIKE ? LIMIT ?, 1",params).unwrap()
        }
        else{
            let params:&[&ToValue] = &[&offset];
            conn.first_exec("SELECT md5(recent_update) FROM v_thread_list LIMIT ?, 1",params).unwrap()
        };
        let mut row = row.unwrap();
        let md5:String = match row.take(0){
            Some(v)=>v,
            None=>String::from("aaaa")
        };
        return md5;
    }
    pub fn upload(conn:&mut mysql::PooledConn, subject:&String, user:User,first_comment:&String)->Result<Self,()>{
        use mysql::IsolationLevel;
        let uid:u32;
        {
            let mut transaction = conn.start_transaction(false, Some(IsolationLevel::Serializable), Some(false)).unwrap();
            let params:&[&ToValue] = &[&user.get_uid(),  subject];
            {
                let result = transaction.prep_exec("INSERT INTO tb_threads (opener_uid,  subject, created_datetime) VALUES (?,?,now())",params).unwrap();
                uid = result.last_insert_id() as u32;
            }
            transaction.prep_exec(r"INSERT INTO tb_comments
                                       (thread_uid, writer_uid, write_datetime, comment)
                                   VALUES
                                       (:thread_uid, :writer_uid, NOW(), :comment)",params!{
                "thread_uid" => uid,
                "writer_uid" => user.get_uid(),
                "comment" => first_comment,
            }).unwrap();
            transaction.commit();
        }
        return match  Self::get(conn, uid){
            Some(v)=>Ok(v),
            None=>Err(())
        };
    }
    pub fn get(conn:&mut mysql::PooledConn, uid:u32)->Option<Self>{
        let sql ="SELECT * FROM v_thread_list WHERE uid = ?";
        let params:&[&ToValue] = &[&uid];
        let row  = conn.first_exec(sql,params).unwrap();
        match row{
            Some(mut row)=>{
                let thread = Thread::new(
                    row.take("uid").expect("uid"),
                    row.take("subject").expect("uid"),
                    row.take("recent_update").expect("recent_update"),
                    row.take("created_datetime").expect("created_datetime"),
                    User::new()
                        .uid(row.take("opener_uid").expect("opener_uid"))
                        .nickname(row.take("opener_nickname").expect("opener_nickname"))
                        .email(row.take("opener_email").expect("opener_email"))
                );
                return Some(thread);
            }, 
            None=>return None
        }
    }
    pub fn delete(self, conn:&mut mysql::PooledConn){
        let sql ="DELETE FROM tb_threads WHERE uid = ?";
        let params:&[&ToValue] = &[&self.get_uid()];
        conn.first_exec(sql,params).unwrap();
    }
    pub fn list(model:&mut mysql::PooledConn ,mut q: Option<String>, offset:usize, count:usize)->Vec<Thread>{
        //let sql =format!("SELECT * FROM v_thread_list WHERE subject like ?");
        let mut sql = String::from("SELECT * FROM v_thread_list ");
        use std::fmt::{Error, Write};
        return if let Some(ref mut q) = q{
            let t = format!("%{}%", q);
            let params:&[&ToValue]= &[&t];
            sql.push_str("WHERE subject LIKE ? ");

            write!(&mut sql, "LIMIT {}, {}", offset, count);
            model.prep_exec(sql,params).unwrap().map(|row|{
            let mut row = row.unwrap();
            Thread::new(
                row.take("uid").expect("uid"),
                row.take("subject").expect("uid"),
                row.take("recent_update").expect("recent_update"),
                row.take("created_datetime").expect("created_datetime"),
                User::new()
                .uid(row.take("opener_uid").expect("opener_uid"))
                .nickname(row.take("opener_nickname").expect("opener_nickname"))
                .email(row.take("opener_email").expect("opener_email"))
            )
            }).collect()
        }
        else{
            let params:&[&ToValue] = &[];
            write!(&mut sql, "LIMIT {}, {}", offset, count);
            model.prep_exec(sql,params).unwrap().map(|row|{
            let mut row = row.unwrap();
            Thread::new(
                row.take("uid").expect("uid"),
                row.take("subject").expect("uid"),
                row.take("recent_update").expect("recent_update"),
                row.take("created_datetime").expect("created_datetime"),
                User::new()
                .uid(row.take("opener_uid").expect("opener_uid"))
                .nickname(row.take("opener_nickname").expect("opener_nickname"))
                .email(row.take("opener_email").expect("opener_email"))
            )
            }).collect()
        };
    }
}