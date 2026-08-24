use super::*;

#[test]
fn tag_parse() {
	use common::HTMLEncode;
	use utils::tag_parse as tag;

	let raw = "[b/タグはちゃんと[i/動作しているか/i]/b]";
	let html = raw.to_html(&tag::Ondyst, false);
	assert_eq!(html, "<b>タグはちゃんと<i>動作しているか</i></b>");
}
