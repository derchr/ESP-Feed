unsafe extern "C" fn event_handler(
    ev: *mut esp_idf_sys::esp_http_client_event,
) -> esp_idf_sys::esp_err_t {
    if (*ev).event_id == esp_idf_sys::esp_http_client_event_id_t_HTTP_EVENT_ON_CONNECTED {
        info!("HTTP connected!");
    }

    if (*ev).event_id == esp_idf_sys::esp_http_client_event_id_t_HTTP_EVENT_ON_DATA {
        let response = std::slice::from_raw_parts((*ev).data as *const u8, (*ev).data_len as usize);
        let string_response = std::str::from_utf8(response).unwrap();
        info!("Response:\n{}\n", string_response);
    }

    esp_idf_sys::ESP_OK as _
}

    // For http.
    // let config = esp_idf_sys::esp_http_client_config_t {
    //     url: url.as_ptr(),
    //     auth_type: esp_idf_sys::esp_http_client_auth_type_t_HTTP_AUTH_TYPE_BASIC,
    //     event_handler: Some(event_handler),
    //     ..Default::default()
    // };


    // http stuff...
    // let client = esp_idf_sys::esp_http_client_init(&config as *const _);
    // esp_idf_sys::esp_http_client_perform(client);
    // esp_idf_sys::esp_http_client_cleanup(client);