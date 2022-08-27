use std::iter::Iterator;
use std::slice::Iter;
use urlencoding::{decode, encode};

#[derive(Debug, Clone)]
pub struct SearchParam(String, Vec<String>);

#[derive(Debug, Clone)]
pub struct SearchParams(Vec<SearchParam>);

impl SearchParams {
    pub fn empty() -> SearchParams {
        SearchParams(vec![])
    }

    pub fn parse(mut s: String) -> Result<SearchParams, ()> {
        if s.starts_with("?") {
            s.remove(0);
        }

        let mut raws = s.split("&").map(|x| String::from(x));
        let mut values = SearchParams::empty();

        while let Some(param) = raws.next() {
            let parsed = SearchParam::parse(param)?;

            let filter = values
                .0
                .iter()
                .filter(|x| x.name() == parsed.name())
                .collect::<Vec<_>>();

            if filter.len() != 0 {
                let index = values
                    .0
                    .iter()
                    .position(|x| x.name() == parsed.name())
                    .unwrap();
                let value = &values.0[index];
                let mut temp = value.1.clone();

                temp.push(parsed.value()[0].clone());

                values.0[index] = SearchParam(value.name().clone(), temp);
            } else {
                values.0.push(parsed);
            }
        }

        Ok(values)
    }

    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        self.0.iter().find(|x| x.name() == name).map(|x| &x.1)
    }

    pub fn push(&mut self, data: SearchParam) {
        let filter = self
            .0
            .iter()
            .filter(|x| x.name() == data.name())
            .collect::<Vec<_>>();

        if filter.len() != 0 {
            let index = self.0.iter().position(|x| x.name() == data.name()).unwrap();
            let value = &self.0[index];
            let mut temp = value.1.clone();

            temp.push(data.value()[0].clone());

            self.0[index] = SearchParam(value.name().clone(), temp);
        } else {
            self.0.push(data);
        }
    }

    pub fn remove(&mut self, name: &str) {
        let index = self.0.iter().position(|x| x.name() == name).unwrap();
        self.0.remove(index);
    }

    pub fn has(&self, name: &str) -> bool {
        self.0.iter().find(|x| x.name() == name).is_some()
    }

    pub fn keys(&self) -> Keys {
        Keys {
            iter: self.0.iter(),
        }
    }

    pub fn values(&self) -> Values {
        Values {
            iter: self.0.iter(),
        }
    }

    pub fn entries(&self) -> Entries {
        Entries {
            iter: self.0.iter(),
        }
    }
}

impl ToString for SearchParams {
    fn to_string(&self) -> String {
        if self.0.len() == 0 {
            return String::from("");
        }

        let mut res = String::from("?");

        for search in self.0.iter() {
            res.push_str(search.to_string().as_str());
            res.push_str("&");
        }

        while res.ends_with("&") {
            res.remove(res.len() - 1);
        }

        res
    }
}

impl ToString for SearchParam {
    fn to_string(&self) -> String {
        let mut res = self.1.iter().fold(String::new(), |acc, cur| {
            acc + format!("{}={}&", encode(self.0.as_str()), encode(cur)).as_str()
        });

        res.remove(res.len() - 1);

        res
    }
}

impl SearchParam {
    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn value(&self) -> &Vec<String> {
        &self.1
    }

    pub fn parse(s: String) -> Result<SearchParam, ()> {
        let split = s.split("=").map(|x| String::from(x)).collect::<Vec<_>>();

        if split.len() != 2 {
            return Err(());
        }

        let mut iter = split.iter();

        let (name, value) = (
            match iter.next() {
                Some(x) => (match decode(x.as_str()) {
                    Ok(y) => y.into_owned(),
                    Err(_) => x.clone(),
                })
                .replace('+', " "),
                None => return Err(()),
            },
            match iter.next() {
                Some(x) => vec![(match decode(x.as_str()) {
                    Ok(y) => y.into_owned(),
                    Err(_) => x.clone(),
                })
                .replace('+', " ")],
                None => return Err(()),
            },
        );

        Ok(SearchParam(name.clone(), value))
    }

    pub fn new(name: String, value: Vec<String>) -> SearchParam {
        SearchParam(name, value)
    }
}

pub struct Keys<'a> {
    iter: Iter<'a, SearchParam>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.name())
    }
}

pub struct Values<'a> {
    iter: Iter<'a, SearchParam>,
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.value())
    }
}

pub struct Entries<'a> {
    iter: Iter<'a, SearchParam>,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<&'a String, &'a Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| Entry(x.name(), x.value()))
    }
}

pub struct Entry<K, V>(K, V);

impl<K, V> Entry<K, V> {
    pub fn key(&self) -> &K {
        &self.0
    }

    pub fn value(&self) -> &V {
        &self.1
    }
}
