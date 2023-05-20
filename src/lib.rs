use std::{fs, io::{self, Read}, collections::{HashSet, HashMap}, hash::{Hash, Hasher}, rc::{Rc, Weak}};
use std::io::Error;

use serde_json::{Value, from_reader};

#[derive(Debug)]
pub enum SchemaError {
    IoError(io::Error),
    Custom {
        name: String,
        description: String
    }
}

impl From<io::Error> for SchemaError {
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

// #[derive(Debug, Clone)]
// pub struct Team(Vec<Worker>);
//
// impl Team {
//     pub fn new() -> Self {
//         Self(vec![])
//     }
//
//     pub fn add_worker(&mut self, worker: Worker) {
//         self.0.push(worker);
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct Worker {
//     name: String,
//     skills: Vec<Skill>
// }
//
// impl Worker {
//     pub fn new(name: &str, skills: &[Skill]) -> Self {
//         Self {
//             name: name.into(),
//             skills: skills.into()
//         }
//     }
//
//     pub fn get_suitable_vacancy(&self) -> Vacancy {
//         todo!()
//     }
// }


// Связать vacancies и skills обычной ссылкой с верменем жизни, а не RC
#[derive(Debug, Clone)]
pub struct CoefficientScheme {
    vacancies: HashSet<Rc<Vacancy>>,
    skills: HashSet<Skill>,
}

impl CoefficientScheme {
    pub fn new(mut schema_f: fs::File) -> Result<Self, SchemaError> {

        let mut schema_bytes = vec![];
        schema_f.read_to_end(&mut schema_bytes)?;

        let json: Value = serde_json::from_slice(&schema_bytes).unwrap();
        //println!("{:?}", json);

        let vacancies = HashSet::from_iter(
            json["vacancies"]
                .as_array()
                .expect("json конфиг не содержит массива в поле 'vacancies'")
                .iter()
                .map(
                    |vacancy| {
                        let vacancy_name_str = vacancy.as_str().unwrap().to_owned();
                        return Rc::new(Vacancy(vacancy_name_str));
                    }
                )
        );

        let mut res_skills = HashSet::default();
        let skills = json["skills"]
            .as_object()
            .expect("json конфиг не содержит объекта в поле 'skills'");

        for (skill_name, vacancies_coef) in skills {

            let vacancies_coef_map = vacancies_coef.as_object().unwrap();
            let mut vacancies_coefficient = Vec::default();

            for (vacancy_name, vacancy_coef) in vacancies_coef_map {

                println!("{:?}", vacancy_coef);
                let vacancy: Vacancy = vacancy_name.clone().into();

                let vacancy_rc = vacancies
                    .get::<Vacancy>(&vacancy)
                    .ok_or(
                        SchemaError::Custom {
                            name: "Not found vacancy in HasSet".to_owned(),
                            description: format!("В хешсете с вакансиями не найдена вакансия: {:?}", vacancy_name)
                        }
                    )?;

                //println!("{:?}", vacancy_rc);
                //println!("{:?}", vacancy_coef);

                vacancies_coefficient.push(VacancyCoefficient(Rc::downgrade(vacancy_rc), vacancy_coef.as_i64().unwrap()));
            }

            res_skills.insert( Skill {
                name: skill_name.clone(),
                vacancies_coefficient
            });
        }

        Ok(Self {
            vacancies,
            skills: res_skills,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vacancy(pub String);


impl Hash for Vacancy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl From<String> for Vacancy {
    fn from(value: String) -> Self {
        Vacancy(value)
    }
}


#[derive(Debug, Clone)]
pub struct Skill {
    name: String,
    vacancies_coefficient: Vec<VacancyCoefficient>
}


impl Hash for Skill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}


impl PartialEq for Skill {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}


impl Eq for Skill {}


#[derive(Debug, Clone)]
pub struct VacancyCoefficient(Weak<Vacancy>, i64);


impl VacancyCoefficient {
    pub fn new(vacancy: Weak<Vacancy>, coefficient: i64) -> Self {
        Self(vacancy, coefficient)
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;

    #[test]
    fn it_works() {

        let f = File::open("../skill_coefficients.json").unwrap();

        let schema = CoefficientScheme::new(f).unwrap();
        println!("\n{:?}", schema);
        println!("\n{:?}", schema.skills.iter().next().unwrap().vacancies_coefficient[0].0.upgrade());
        assert_eq!(4, 4);
    }
}
