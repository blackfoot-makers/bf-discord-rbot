macro_rules! db_add {
  ($name:ident,$new:ident, $result:ident, $table:ident ) => {
    pub fn $name(&mut self, new: $new) {
      let result: $result = diesel::insert_into($table::table)
        .values(&new)
        .get_result(&self.get_connection())
        .expect("Error saving new $table");
      self.$table.push(result);
    }
  };
}

macro_rules! hashmap {
  ($( $key: expr => $val: expr ),*) => {{
       let mut map = ::std::collections::HashMap::new();
       $( map.insert($key, $val); )*
       map
  }}
}

macro_rules! db_load {
  ($name:ident, $result:ident, $table:ident ) => {
    pub fn $name(&mut self) {
      use super::schema::$table::dsl::*;

      let results = $table
        .load::<$result>(&self.get_connection())
        .expect("Error loading $table");

      self.$table = results;
    }
  };
}

macro_rules! closure_call_async {
  ($function:expr ) => {
    |args| -> CallbackReturn { blocking($function(args)) }
  };
}
