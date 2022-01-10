use embedded_svc::storage::Storage;

    // info!("Open storage!");
    // let mut nvs_storage =
    //     esp_idf_svc::nvs_storage::EspNvsStorage::new_default(default_nvs, "esp_feed", true)
    //         .expect("Failed to open NVS storage.");

    // if let Ok(vec) = nvs_storage.get_raw("my_key") {
    //     println!("{:?}", vec);
    // }

    // let my_obj = MyObject {
    //     data: "Hallo Welt!".into(),
    //     len: 12,
    //     flags: Box::new(0x100),
    //     valid: Some(true),
    //     world: Matter::Air(5.123213),
    // };

    // nvs_storage.put("my_key", &my_obj).unwrap_or_else(|e| {
    //     warn!("Could not store key: {:?}", e);
    //     false
    // });

    // if let Ok(vec) = nvs_storage.get_raw("my_key") {
    //     println!("{:?}", vec);
    //     println!("{}", std::str::from_utf8(&(vec.unwrap())).unwrap());
    // }

    // if let Ok(e) = nvs_storage.get("my_key") {
    //     let s: MyObject = e.unwrap();
    //     println!("Value: {:?}", s);
    // }