use proc_macro::TokenStream;

mod metadata_table;
mod from_byte_stream;

#[proc_macro_derive(MetadataTable, attributes(table_index, heap_index, coded_index))]
pub fn metadata_table(ast: TokenStream) -> TokenStream {
	let ast = syn::parse(ast).unwrap();
	metadata_table::derive(ast)
}

#[proc_macro_derive(FromByteStream, attributes(check_value, validate_value))]
pub fn from_byte_stream(ast: TokenStream) -> TokenStream {
	let ast = syn::parse(ast).unwrap();
	from_byte_stream::derive(ast)
}
