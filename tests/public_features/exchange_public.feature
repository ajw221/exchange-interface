# Feature: Exchange with Public and Private APIs
Feature: Exchange Public API Endpoints 
    Background: 
        Given an exchange instance
        When API_KEY, API_SECRET, and BASE_URL exist
        Then the exchange instance keys are populated

    Scenario: Using a valid exchange instance, retrieve the server time and validate the response
        Given a server time request is sent
        When a server time response is received
        Then the response time in minutes should equal the server time in minutes
        
    Scenario: Using a valid exchange instance, retrieve the XBT/USD trading pair and validate the response
        Given a XBT/USD trading pair request is sent
        When a XBT/USD trading pair response is received 
        Then the response should contain XBT/USD asset pair information

