use std::collections::BTreeSet;
use url::Url;

pub(super) fn doi() -> String {
    "10.1145/3428204".to_string()
}

pub(super) fn title() -> String {
    "形状记忆聚氨酯的合成及其在织物中的应用".to_string()
}

pub(super) fn abstraction() -> String {
    "（摘要是一篇具有独立性和完整性的短文，应概括而扼要地反映出本论文的主要内容。\
    包括研究目的、研究方法、研究结果和结论等，特别要突出研究结果和结论。\
    中文摘要力求语言精炼准确，博士学位论文建议1000~1200字，硕士学位论文摘要建议500~800字。\
    摘要中不可出现参考文献、图、表、化学结构式、非公知公用的符号和术语。\
    英文摘要与中文摘要的内容应完全一致，在语法、用词上应准确无误，语言简练通顺。\
    留学生的英文版博士学位论文中应有不少于3000字的“详细中文摘要”。）".to_string()
}

pub(super) fn keywords() -> Vec<String> {
    vec!["形状记忆".to_string(), "聚氨酯".to_string(), "织物".to_string(), "合成".to_string(), "应用".to_string()]
}

pub(super) fn language() -> BTreeSet<String> {
    BTreeSet::from(["zh_CN".to_string(), "en_US".to_string()])
}

pub(super) fn homepage() -> Option<Url> {
    Some(Url::parse("https://bithesis.bitnp.net/").unwrap())
}

pub(super) fn template_link() -> Option<Url> {
    Some(Url::parse("https://github.com/BITNP/BIThesis/releases").unwrap())
}

pub(super) fn community_link() -> Option<Url> {
    Some(Url::parse("https://jq.qq.com/?_wv=1027&k=KYDrmS5z").unwrap())
}

pub(super) fn profile_name() -> String {
    "納西妲".to_string()
}
