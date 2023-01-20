#[macro_export]
macro_rules! __impl_multi_row_table {
    (
		$Name: ident $(,)?

		Row($self: ident, $reader_name: ident) {
			$($row_field: ident : $row_ty: ty = $row_expr: expr),*
		} $(,)?

		Table($tables_name: ident) {
			row_size = $calc_row_size: block

			$($table_field: ident : $table_ty: ty = $table_expr: expr),*
		} $(,)?
	) => {
		paste! {
			#[derive(Debug, Clone)]
			pub struct $Name {
				$($row_field: $row_ty),*
			}

			#[derive(Clone)]
			pub struct [<$Name Table>]<'l> {
				bytes: &'l [u8],
				row_size: usize,
				$($table_field: $table_ty),*
			}

			#[derive(Clone)]
			pub struct [<$Name Iterator>]<'l> {
				reader: ByteStream<'l>,
				table: [<$Name Table>]<'l>,
			}

			impl <'l> MetadataTable<'l> for [<$Name Table>]<'l> {
				type Iter = [<$Name Iterator>]<'l>;

				fn bytes(&self) -> &'l [u8] {
					self.bytes
				}

				fn row_size(&self) -> usize {
					self.row_size
				}

				fn iter(&self) -> Self::Iter {
					Self::Iter {
						table: self.clone(),
						reader: ByteStream::new(self.bytes),
					}
				}
			}

			impl ParseRow for [<$Name Table>]<'_> {
				type Row = $Name;

				fn parse_row(&$self, $reader_name: &mut ByteStream) -> Result<Self::Row, Error> {
					Ok(Self::Row {
						$($row_field: $row_expr),*
					})
				}
			}

			impl <'l> MetadataTableImpl<'l> for [<$Name Table>]<'l> {
				fn cli_identifier() -> TableKind {
					TableKind::$Name
				}

				fn calc_row_size($tables_name: &TableHeap) -> usize {
					$calc_row_size
				}

				fn new(bytes: &'l [u8], $tables_name: &TableHeap) -> Result<Self, Error> {
					Ok(Self {
						bytes,
						row_size: Self::calc_row_size($tables_name),
						$($table_field: $table_expr),*
					})
				}
			}

			impl Iterator for [<$Name Iterator>]<'_> {
				type Item = Result<$Name, Error>;

				fn next(&mut self) -> Option<Self::Item> {
					match self.reader.remaining() {
						0 => None,
						_ => Some(self.table.parse_row(&mut self.reader)),
					}
				}
			}

			impl $Name {
				$(
					pub fn $row_field(&self) -> $row_ty {
						self.$row_field
					}
				)*
			}
		}
	};
}
