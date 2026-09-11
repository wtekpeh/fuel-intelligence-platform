use crate::drivers::Modem;

pub async fn run_network_diagnostics(modem: &mut Modem<'_>) {
    modem
        .send_command_and_print_response_async(b"AT\r\n", "AT")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+CPIN?\r\n", "AT+CPIN?")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+CSQ\r\n", "AT+CSQ")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+CEREG?\r\n", "AT+CEREG?")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+CGATT?\r\n", "AT+CGATT?")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+CGPADDR\r\n", "AT+CGPADDR")
        .await;
}
