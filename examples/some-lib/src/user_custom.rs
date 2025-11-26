use gpui::{App, Div, IntoElement, Stateful, Window};
use gpui_component::table::{Column, TableDelegate};
use gpui_table::{NamedTableRow, TableRowMeta as _, TableRowStyle as _};

#[derive(NamedTableRow)]
#[table(id = "users", title = "Users", delegate = false)]
pub struct UserCustom {
    #[table(col = "id", title = "ID", width = 60.)]
    id: usize,

    #[table(col = "name", title = "Name", sortable, width = 150.)]
    name: String,

    #[table(col = "age", title = "Age", sortable, width = 80.)]
    age: u32,

    #[table(col = "email", title = "Email", width = 200.)]
    email: String,

    #[table(col = "active", title = "Active", width = 50.)]
    active: bool,
}

pub struct UsersCustomTableDelegate {
    rows: Vec<UserCustom>,
}

impl TableDelegate for UsersCustomTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        UserCustom::table_columns().len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> &Column {
        &UserCustom::table_columns()[col_ix]
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        self.rows[row_ix].render_table_cell(col_ix, window, cx)
    }

    fn render_tr(&self, row_ix: usize, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        self.rows[row_ix].render_table_row(row_ix, window, cx)
    }
}
