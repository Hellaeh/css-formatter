use radix::Radix;

#[test]
fn double_free() {
	struct Idk(Box<usize>);

	impl Idk {
		fn new(num: usize) -> Self {
			Self(Box::new(num))
		}
	}

	let mut radix = Radix::new();

	assert!(radix.get("aaa").is_none());
	radix.insert("aaa", Idk::new(1));

	assert!(radix.get("aaaa").is_none());
	radix.insert("aaaa", Idk::new(2));

	assert!(radix.get("aaa").is_some());
	assert!(radix.get("aaaa").is_some());

	drop(radix);

	println!("ignore this print statement");
}

#[test]
fn simple() {
	let mut radix = Radix::new();

	radix.insert("background-color", 108);
	radix.insert("background", 101);
	radix.insert("backpack", 102);
	radix.insert("backpack1", 104);
	radix.insert("aaa", 103);
	radix.insert("aaaa", 104);
	radix.insert("aa", 105);
	radix.insert("a", 106);
	radix.insert("opacity", 1);
	radix.insert("overflow", 2);
	radix.insert("overflow-x", 3);
	radix.insert("overflow-y", 4);
	radix.insert("to", 13);
	radix.insert("top", 10);
	radix.insert("tou", 11);
	radix.insert("tos", 12);
	dbg!(&radix);

	assert_eq!(radix.get("tos"), Some(&12));
	assert_eq!(radix.get("tou"), Some(&11));
	assert_eq!(radix.get("top"), Some(&10));
	assert_eq!(radix.get("to"), Some(&13));
	assert_eq!(radix.get("opacity"), Some(&1));
	assert_eq!(radix.get("overflow"), Some(&2));
	assert_eq!(radix.get("overflow-x"), Some(&3));
	assert_eq!(radix.get("overflow-y"), Some(&4));
	assert_eq!(radix.get("a"), Some(&106));
	assert_eq!(radix.get("aa"), Some(&105));
	assert_eq!(radix.get("aaa"), Some(&103));
	assert_eq!(radix.get("aaaa"), Some(&104));
	assert_eq!(radix.get("aaaaa"), None);
	assert_eq!(radix.get("background"), Some(&101));
	assert_eq!(radix.get("background-color"), Some(&108));
	assert_eq!(radix.get("background2"), None);
	assert_eq!(radix.get("backgroun"), None);
	assert_eq!(radix.get("backpack"), Some(&102));
	assert_eq!(radix.get("aaaa"), Some(&104));
}
