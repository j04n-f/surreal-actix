DEFINE TABLE OVERWRITE account SCHEMAFULL;

DEFINE FIELD OVERWRITE name ON TABLE account TYPE string ASSERT string::len($value) > 2;
DEFINE FIELD OVERWRITE email ON TABLE account TYPE string ASSERT string::is::email($value);
DEFINE FIELD OVERWRITE password ON TABLE account TYPE string PERMISSIONS FOR select NONE;
DEFINE FIELD OVERWRITE created_at ON account VALUE $before OR time::now() DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON account VALUE time::now() DEFAULT time::now();

DEFINE INDEX OVERWRITE unique_email ON TABLE account COLUMNS email UNIQUE;
