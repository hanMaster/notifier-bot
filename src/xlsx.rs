use crate::Result;
use rust_xlsxwriter::*;
use crate::adapters::mailer::data_types::DealInfo;

pub struct Xlsx;

impl Xlsx {
    pub fn create(deals: Vec<DealInfo>) -> Result<Vec<u8>> {
        // Create a new Excel file object.
        let mut workbook = Workbook::new();

        // Create some formats to use in the worksheet.
        let header_format = Format::new().set_bold().set_align(FormatAlign::Center);
        let row_format = Format::new().set_align(FormatAlign::Center);
        // let date_format = Format::new().set_num_format("dd.mm.yyyy");

        // Add a worksheet to the workbook.
        let worksheet = workbook.add_worksheet();

        // Set the column width for clarity.
        worksheet.set_column_width(0, 15)?;
        worksheet.set_column_width(1, 10)?;
        worksheet.set_column_width(2, 15)?;
        worksheet.set_column_width(3, 22)?;
        worksheet.set_column_width(4, 22)?;
        worksheet.set_column_width(5, 22)?;
        worksheet.set_column_width(6, 22)?;

        // Write a string without formatting.
        worksheet.write_with_format(0, 0, "Проект", &header_format)?;
        worksheet.write_with_format(0, 1, "Дом", &header_format)?;
        worksheet.write_with_format(0, 2, "Тип объекта", &header_format)?;
        worksheet.write_with_format(0, 3, "Номер объекта", &header_format)?;
        worksheet.write_with_format(0, 4, "Тип отделки", &header_format)?;
        worksheet.write_with_format(0, 5, "Дата регистрации", &header_format)?;
        worksheet.write_with_format(0, 6, "Передать объект до", &header_format)?;
        
        for (idx, deal) in deals.iter().enumerate() {
            worksheet.write_with_format((idx + 1) as RowNum, 0, &deal.project, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 1, deal.house, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 2, &deal.object_type, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 3, deal.object, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 4, &deal.facing, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 5, &deal.reg_date, &row_format)?;
            worksheet.write_with_format((idx + 1) as RowNum, 6, &deal.exp_date, &row_format)?;
        }

        // // Write a date.
        // let date = ExcelDateTime::parse_from_str("2023-1-25")?;
        // worksheet.write_with_format(0, 7, &date, &date_format)?;

        // Save the file to disk.
        let buf = workbook.save_to_buffer()?;

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_worksheet() {
        Xlsx::create(vec![]).unwrap();
    }
}
