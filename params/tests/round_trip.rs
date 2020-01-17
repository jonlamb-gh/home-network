use params::*;

#[test]
fn round_trip_list_all() {
    let mut bytes = vec![0xFF; 1500];

    let tx_req = Request::new(0, 0, GetSetOp::ListAll, GetSetPayloadType::None);
    let wire_size = tx_req.wire_size();
    let mut frame = GetSetFrame::new_unchecked(&mut bytes[..wire_size]);
    assert_eq!(tx_req.emit(&mut frame), Ok(()));

    let frame = GetSetFrame::new_checked(&bytes[..wire_size]).unwrap();
    assert_eq!(frame.op(), GetSetOp::ListAll);
    let rx_req = Request::parse(&frame).unwrap();
    assert_eq!(rx_req, tx_req);

    let p_a = Parameter::new_with_value(
        ParameterId::new(0x0A),
        ParameterFlags(0),
        ParameterValue::I32(-1234),
    );
    let p_b = Parameter::new_with_value(
        ParameterId::new(0x0B),
        ParameterFlags(0),
        ParameterValue::Bool(true),
    );

    let mut tx_resp = Response::new(0, 0, GetSetOp::ListAll);
    assert_eq!(tx_resp.push(p_a), Ok(()));
    assert_eq!(tx_resp.push(p_b), Ok(()));
    let wire_size = tx_resp.wire_size();
    let mut frame = GetSetFrame::new_unchecked(&mut bytes[..wire_size]);
    assert_eq!(tx_resp.emit(&mut frame), Ok(()));

    let frame = GetSetFrame::new_checked(&bytes[..wire_size]).unwrap();
    assert_eq!(frame.op(), GetSetOp::ListAll);
    let rx_resp = Response::parse(&frame).unwrap();
    assert_eq!(rx_resp, tx_resp);
}
