use quick_xml::events::{BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;
use url::Url;

pub async fn get_mybangumi_rss(token: &str) -> Result<String, ()> {
    let url = format!("https://mikan.proxy.pch.pub/RSS/MyBangumi?token={}", token);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "bangumi-rss-proxy")
        .send()
        .await
        .map_err(|_| ())?;
    let resp_text = resp.text().await.map_err(|_| ())?;
    Ok(resp_text)
}

pub async fn edit_mybangumi_rss(raw_rss_data: &str, domain: &str) -> Result<String, ()> {
    let mut reader = Reader::from_str(raw_rss_data);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut labels: Vec<(String, Vec<(_, _)>)> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                {
                    let mut attributes: Vec<_> = vec![];
                    for attr in e.attributes() {
                        let a = attr.unwrap();
                        attributes.push((
                            String::from_utf8(a.key.as_ref().to_vec()).unwrap(),
                            String::from_utf8(a.value.to_vec()).unwrap(),
                        ));
                    }

                    labels.push(
                        (
                            String::from_utf8(e.name().as_ref().to_vec())
                                .unwrap_or("default labal".to_string()),
                            attributes,
                        )
                            .to_owned(),
                    );
                }
                writer.write_event(Event::Start(e)).or(Err(()))?;
            }
            Ok(Event::End(e)) => {
                labels.pop();

                writer.write_event(Event::End(e)).or(Err(()))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                // <enclosure type="application/x-bittorrent"></enclosure>
                if labels
                    .iter()
                    .map(|labal| labal.0.to_owned())
                    .collect::<Vec<String>>()
                    == ["rss", "channel", "item"]
                {
                    let e_raw = e.to_vec();
                    let e_raw = String::from_utf8(e_raw).unwrap();
                    let e_raw_text_single = e_raw
                        .char_indices()
                        .map(|word| word.1)
                        .collect::<Vec<char>>();
                    let mut e_type_vec = vec![];
                    let mut e_text_vec: Vec<(String, String)> = vec![];
                    let mut index = 0;
                    for i in 0..e_raw_text_single.len() {
                        if e_raw_text_single[i] != ' ' {
                            e_type_vec.push(e_raw_text_single[i]);
                        } else {
                            index = i + 1;
                            break;
                        }
                    }
                    let e_type_string = e_type_vec.iter().collect::<String>();
                    if e_type_string != "enclosure" {
                        writer.write_event(e).or(Err(()))?;
                    } else {
                        let mut e_text_temp: Vec<String> = vec![];
                        let mut e_text_temp_child: Vec<char> = vec![];
                        let mut i = index;
                        let mut depth = 0;
                        loop {
                            if i >= e_raw_text_single.len() {
                                break;
                            }
                            if depth == 0 && e_raw_text_single[i] == ' ' {
                                i += 1;
                                continue;
                            } else if depth == 0 && e_raw_text_single[i] == '=' {
                                e_text_temp.push(e_text_temp_child.iter().collect::<String>());
                                e_text_temp_child = vec![];
                                i += 1;
                            } else if depth == 0 && e_raw_text_single[i] != '"' {
                                e_text_temp_child.push(e_raw_text_single[i]);
                                i += 1;
                                continue;
                            } else if depth == 0 && e_raw_text_single[i] == '"' {
                                i += 1;
                                depth = 1;
                                continue;
                            } else if depth == 1 && e_raw_text_single[i] == '\\' {
                                e_text_temp_child.push(e_raw_text_single[i]);
                                e_text_temp_child.push(e_raw_text_single[i + 1]);
                                i += 2;
                            } else if depth == 1
                                && e_raw_text_single[i] != '"'
                                && e_raw_text_single[i] != '\\'
                            {
                                e_text_temp_child.push(e_raw_text_single[i]);
                                i += 1;
                                continue;
                            } else if depth == 1 && e_raw_text_single[i] == '"' {
                                e_text_temp.push(e_text_temp_child.iter().collect::<String>());
                                e_text_vec
                                    .push((e_text_temp[0].to_owned(), e_text_temp[1].to_owned()));
                                e_text_temp = vec![];
                                e_text_temp_child = vec![];
                                depth = 0;
                                i += 1;
                            } else if depth == 0 && e_raw_text_single[i] == '"' {
                                depth = 1;
                                i += 1;
                                continue;
                            }
                        }
                        // 替换链接
                        let mut e_text_vec_new: Vec<(String, String)> = vec![];
                        for i in e_text_vec {
                            if i.0 == "url" {
                                let url = Url::parse(&i.1).or(Err(()))?;
                                let path = url.path();
                                let query = url.query();
                                let new_url = match query {
                                    Some(query) => {
                                        format!("{}{}?{}", domain, path, query)
                                    }
                                    None => {
                                        format!("{}{}", domain, path)
                                    }
                                };

                                e_text_vec_new.push((i.0, new_url));
                            } else {
                                e_text_vec_new.push(i);
                            }
                        }
                        let mut e_text_vec_new_string = String::new();
                        for i in e_text_vec_new {
                            e_text_vec_new_string.push_str(&format!("{}=\"{}\" ", i.0, i.1));
                        }
                        let e_text_vec_new_string = e_text_vec_new_string.trim_end();
                        let e_text_vec_new_string =
                            format!("<enclosure {} />", e_text_vec_new_string);
                        let e_text_vec_new_string = BytesText::from_escaped(e_text_vec_new_string);
                        let e_text_vec_new_string = Event::Text(e_text_vec_new_string);
                        writer.write_event(e_text_vec_new_string).or(Err(()))?;
                    }
                } else {
                    writer.write_event(e).or(Err(()))?;
                }
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let result = writer.into_inner().into_inner();
    let result = String::from_utf8(result).or(Err(()))?;
    Ok(result)
}
