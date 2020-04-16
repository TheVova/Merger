# Merger
A rust proc macro to allow merging/updating structs. This crate provides a trait 'MergeFrom', and a derive macro of the same name.
The behaviour currently mimics that of protobuf's MergeFrom:
- Scalar values get replaced.
- 'Repeated' values (`Vec`,`String`) append.
- Maps (`BTreeMap`,`HashMap`) recursively merge fields that exist in both, or insert.
- Enums are supported for unit variants or variants with 1 field. unit variants work like scalars (replaced), while variants with a field either merge the field (if same variant in both 'to' and 'from') or replace (if different variants in 'to' and 'from')

## TODOs
- Add support for specifying custom merge function (for example if you'd like `Vec` to also replace).
- Add support for ignoring fields (useful if you have fields that should only be set explicitly).

## Contributing
Yes please.
