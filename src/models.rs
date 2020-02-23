use super::schema::rigs;

#[derive(Queryable)]
pub struct Rig {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub active: bool,
}

#[derive(Insertable)]
#[table_name = "rigs"]
pub struct NewRig<'a> {
    pub title: &'a str,
    pub body: &'a str,
}
