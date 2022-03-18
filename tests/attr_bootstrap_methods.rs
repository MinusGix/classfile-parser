extern crate classfile_parser;
extern crate nom;

use classfile_parser::attribute_info::bootstrap_methods_attribute_parser;
use classfile_parser::class_parser;
use classfile_parser::constant_info::ConstantInfo;
use classfile_parser::constant_pool::ConstantPoolIndexRaw;
use classfile_parser::parser::ParseData;

#[test]
fn test_attribute_bootstrap_methods() {
    let class_file_data: &[u8] =
        include_bytes!("../java-assets/compiled-classes/BootstrapMethods.class");
    match class_parser(ParseData::new(class_file_data)) {
        Result::Ok((_, c)) => {
            println!("Valid class file, version {:?}, const_pool({}), this=const[{:?}], super=const[{:?}], interfaces({}), fields({}), methods({}), attributes({}), access({:?})", c.version, c.const_pool_size, c.this_class, c.super_class, c.interfaces_count, c.fields_count, c.methods_count, c.attributes_count, c.access_flags);

            let mut bootstrap_method_const_index = 0;

            println!("Constant pool:");
            for (const_index, const_item) in c.const_pool.iter().enumerate() {
                println!("\t[{}] = {:?}", (const_index + 1), const_item);
                if let ConstantInfo::Utf8(ref c) = *const_item {
                    if c.as_text(class_file_data) == "BootstrapMethods" {
                        if bootstrap_method_const_index != 0 {
                            panic!("Should not find more than one BootstrapMethods constant");
                        }
                        bootstrap_method_const_index = (const_index + 1) as u16;
                    }
                }
            }
            assert_ne!(bootstrap_method_const_index, 0);

            println!(
                "Bootstrap Methods constant index = {}",
                bootstrap_method_const_index
            );

            for (_, attribute_item) in c.attributes.iter().enumerate() {
                if attribute_item.attribute_name_index.0 == bootstrap_method_const_index {
                    match bootstrap_methods_attribute_parser(ParseData::from_range(
                        class_file_data,
                        attribute_item.info.clone(),
                    )) {
                        Result::Ok((_, bsma)) => {
                            assert_eq!(bsma.num_bootstrap_methods, 1);
                            let bsm = &bsma.bootstrap_methods[0];
                            assert_eq!(bsm.bootstrap_method_ref, ConstantPoolIndexRaw::new(36));

                            println!("{:?}", bsm);
                            println!("\tmethod ref: {:?}", c.const_pool.get(36));
                            println!("\t\tdescriptor: {:?}", c.const_pool.get(53));
                            println!("\t\t\tclass_index: {:?}", c.const_pool.get(9));
                            println!("\t\t\t\tname_index: {:?}", c.const_pool.get(51));
                            println!("\t\t\t\t\tclass_index: {:?}", c.const_pool.get(64));
                            println!("\t\t\t\t\tname_and_type_index: {:?}", c.const_pool.get(65));
                            println!("\t\t\t\t\t\tname_index: {:?}", c.const_pool.get(30));
                            println!("\t\t\t\t\t\tdescriptor_index: {:?}", c.const_pool.get(31));
                            println!("\t\t\tname_and_type_index: {:?}", c.const_pool.get(66));
                            return;
                        }
                        _ => panic!("Failed to parse bootstrap method attribute"),
                    }
                }
            }

            panic!("Should not get to here");
        }
        _ => panic!("Not a valid class file"),
    }
}

#[test]
fn should_have_no_bootstrap_method_attr_if_no_invoke_dynamic() {
    let class_file_data: &[u8] = include_bytes!("../java-assets/compiled-classes/BasicClass.class");
    match class_parser(ParseData::new(class_file_data)) {
        Result::Ok((_, c)) => {
            for (_, const_item) in c.const_pool.iter().enumerate() {
                if let ConstantInfo::Utf8(ref c) = *const_item {
                    if c.as_text(class_file_data) == "BootstrapMethods" {
                        panic!("Should not have found a BootstrapMethods constant in a class not requiring it")
                    }
                }
            }
        }
        _ => panic!("Not a valid class file"),
    }
}
