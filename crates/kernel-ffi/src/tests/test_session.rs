    #[test]
    fn runtime_contract_invalid_curve_degree_matches_bindings_error_status() {
        let session = create_session();
        let points = runtime_curve_points();
        let mut object = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                8,
                false,
                tol(),
                &mut object as *mut _,
            ),
            RgmStatus::InvalidInput
        );

        let mut error_code = 0_i32;
        assert_eq!(
            rgm_last_error_code(session, &mut error_code as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(error_code, RgmStatus::InvalidInput as i32);

        let mut error_msg_buf = [0_u8; 256];
        let mut error_msg_len = 0_usize;
        assert_eq!(
            rgm_last_error_message(
                session,
                error_msg_buf.as_mut_ptr(),
                error_msg_buf.len(),
                &mut error_msg_len as *mut _,
            ),
            RgmStatus::Ok
        );
        let error_message = std::str::from_utf8(&error_msg_buf[..error_msg_len]).unwrap_or("");
        assert!(error_message.contains("fit points"));

        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn session_create_and_destroy() {
        let mut session = RgmKernelHandle(0);
        assert_eq!(rgm_kernel_create(&mut session as *mut _), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::NotFound);
    }

    #[test]
    fn alloc_and_dealloc_roundtrip() {
        let mut ptr: *mut u8 = ptr::null_mut();
        assert_eq!(rgm_alloc(64, 8, &mut ptr as *mut _), RgmStatus::Ok);
        assert!(!ptr.is_null());

        // SAFETY: ptr is allocated for 64 bytes above.
        unsafe {
            for idx in 0..64 {
                *ptr.add(idx) = idx as u8;
            }
        }

        assert_eq!(rgm_dealloc(ptr, 64, 8), RgmStatus::Ok);
    }

    #[test]
    fn threaded_sessions_are_isolated() {
        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let session = create_session();
                    let points = sample_points();
                    let mut object = RgmObjectHandle(0);
                    let status = rgm_nurbs_interpolate_fit_points(
                        session,
                        points.as_ptr(),
                        points.len(),
                        2,
                        false,
                        tol(),
                        &mut object as *mut _,
                    );
                    assert_eq!(status, RgmStatus::Ok);
                    assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
                    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
                })
            })
            .collect();

        for thread in threads {
            thread.join().expect("thread should complete");
        }
    }
