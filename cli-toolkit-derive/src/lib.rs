use proc_macro::TokenStream;

mod metadata_table;

#[proc_macro_derive(MetadataTable, attributes(table_index, heap_index, coded_index))]
pub fn derive_metadata_table(ast: TokenStream) -> TokenStream {
	let ast = syn::parse(ast).unwrap();
	metadata_table::derive(ast)
}
