Feature: Exchange Private API Endpoints using 2FA
    Background: 
        Given an exchange instance
        When API_KEY_2FA, API_SECRET_2FA, and BASE_URL exist
        Then the exchange instance keys are populated

    Scenario: If API_PASSPHRASE exists in the environment variables, test each example's signing data
        Given API_PASSPHRASE exists
        When using <private_key>, <nonce>, and <endpoint> for sign testing
        Then the resulting value should be equal to <signed>

        Examples:
            | private_key | nonce | endpoint      | signed                                                                                   |
            | 11111111    | 12345 | /test-route-1 | kiokL1J/JcwAqnbpekTnIlQgcmCrFcRcLkalCwo82Xk2eljKD1XxMrUXpiyX1zRXIoM5BGMD+VZCFJ30hkukGA== |
            | ijklmnop    | 67890 | /test-route-2 | /43JHatvokXs2Vp9Per4FP8uBtNNGT6ICbxz112L6vkSHOvZ6YD9+XWJAYxfzKgmGcNsbiwT8jLKqmrNadrcKw== |
            | abcdefgh    | 54321 | /test-route-3 | /D0mmIQeTY1o4Lr11ZVw6Yvg6SVNP9Iq39ky2mvguQXE3fxe1ZlJDnADDpEQq3xV6hSyuE5ekiH4PuYDViMLkw== |

    Scenario: Using a valid exchange instance requiring 2FA, retrieve open orders and validate the content
        Given a populated exchange instance requires API_PASSPHRASE
        When an open orders request is sent and a response is received with 0 errors
        Then the response should contain the OpenOrders result
