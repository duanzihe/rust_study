//! # dzh_crate
//! 
//!  crate.io  的测试,此包实现了“读取命令行输入，查找对应路径文件中文本拥有对应字段的行，并打印输出。”
///  # ExampleS
/// 
///  '''
///let query = "duct";
/// let contents = "\
///     Rust:
///     safe, fast, productive.
///     Pick three.";
/// assert_eq!(vec!["safe, fast, productive."], search(query, contents));
/// '''

use std::error::Error;
use std::fs;
use std::env;
pub struct Config{
    pub query:String,
    pub file_path:String,
    pub ignore_case: bool,
}
// impl Config {
//     pub fn build(args: &[String]) -> Result<Config, &'static str> {
//         if args.len() < 3 {
//             return Err("not enough arguments");
//         }

//         let query = args[1].clone();
//         let file_path = args[2].clone();
//         let ignore_case = env::var("IGNORE_CASE").is_ok();
//         Ok(Config { query, file_path,ignore_case })
//     }
// }
impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>,  //参数是一个可变且实现了string迭代器的东西
    ) -> Result<Config, &'static str> {  //返回值是一个oresult,成功是config,失败了就是错误信息
        args.next();

        let query = match args.next() { //对迭代器中的下一个值,因为这个值是option进行模式匹配
            Some(arg) => arg,//如果有值，就返回一个arg
            None => return Err("Didn't get a query string"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };
        //检查环境变量是否存在ignore_case=1,如果有则设置bool为真，意思就是忽略大小写。
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {   //如果都没错，就用传入的这些值创建一个config实例并返回
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config:&Config) ->Result<(),Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?; //正常会从配置路径中读取相关文件的内容，如果错误 就会在这里return错误信息

    if config.ignore_case {   //检查环境变量，并分别查找大小写是否敏感的字段。
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };


    Ok(())   //如果一直顺利执行就返回一个空的ok值
}
///  # ExampleS
/// 
///  ```
/// let query = "duct";
/// let contents = "\
///     Rust:
///     safe, fast, productive.
///     Pick three.";
/// assert_eq!(vec!["safe, fast, productive."], minigrep::search(query, contents));
/// ```
///为了满足需求，我需要设计一个search函数，把他放到run中,让他获取查询字段和文本，以一个拥有所有权的string向量，返回包含查询字段的行。.

pub fn  search (query:&str,contents:&str)-> Vec<String>{//为了满足需求，我需要设计一个search函数，把他放到run中,让他获取查询字段和文本，以一个拥有所有权的string向量，返回包含查询字段的行。
    let lines_of_contents:Vec<String>=contents.lines()
    .filter(|line| line.contains(query))
    .map(|line| line.trim().to_string()).collect(); // 使用 trim() 移除每行开头和结尾的所有空白字符,然后用to_string来创建一个新的String实例，最后collect起来变成vec。
    println!("{:?}",lines_of_contents);
    lines_of_contents
}

pub fn search_case_insensitive(query: &str,contents: &str,) -> Vec<String> {
    let lines_of_contents:Vec<String>=contents.lines()
    .filter(|line|line.to_lowercase().contains(&query.to_lowercase()))
    .map(|line| line.trim().to_string()).collect();
    println!("{:?}",lines_of_contents);
    lines_of_contents
}

//为测试驱动开发准备模块
#[cfg(test)]  //配置属性，表示一段代码只在测试时编译
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
            Rust:
            safe, fast, productive.
            Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_unsensitive(){
        let query = "rUsT";
        let contents = "\
            Rust:
            safe, fast, productive.
            Pick three.
            Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }       
}
