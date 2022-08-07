Feature: Private API Endpoints without 2FA
    Background:
        Given an exchange instance
        When API_KEY, API_SECRET, and BASE_URL exist
        Then the exchange instance keys are populated

    Scenario: Using a valid exchange instance not requiring 2FA, retrieve open orders and validate the content
        Given a populated exchange instance not requiring API_PASSPHRASE
        When an open orders request is sent and a response is received with 0 errors
        Then the response should contain the OpenOrders result