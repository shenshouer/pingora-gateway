use wildcard::Wildcard;

fn main() {
    // 匹配*
    let wildcard = Wildcard::new("*.example.com".as_bytes()).unwrap();
    assert!(wildcard.is_match("*.example.com".as_bytes()));
    assert!(wildcard.is_match("www.example.com".as_bytes()));

    // // 没有任何通配符
    // let wildcard = Wildcard::new("abc.example.com".as_bytes()).unwrap();
    // assert!(wildcard.is_match("abc.example.com".as_bytes()));

    // 提取通配符
    let captures: Vec<&[u8]> = wildcard.captures("abc.example.com".as_bytes()).unwrap();
    assert_eq!(captures, vec![b"abc"]);

    let captures: Vec<&[u8]> = wildcard.captures("*.example.com".as_bytes()).unwrap();
    assert_eq!(captures, vec![b"*"]);
}
