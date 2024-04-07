import {type Configuration, LogLevel, type PopupRequest, PublicClientApplication} from "@azure/msal-browser";

const config: Configuration = {
    // required
    auth: {
        // must match info in dashboard
        clientId: "a4b8584b-9fbd-4bc0-bbfb-363589f1743b",
        authority: "https://login.microsoftonline.com/b2748d0a-856e-4184-bda8-831f9ffa8a48/v2.0",

        // login redirect; must match path in dashboard
        redirectUri: `${location.origin}/`,
    },

    // optional
    system: {
        loggerOptions: {
            logLevel: LogLevel.Error,
            loggerCallback,
        }
    }
}

function loggerCallback(level: LogLevel, message: string, containsPii: boolean) {
    if (!containsPii) {
        const parts = message.split(' : ')
        const text = parts.pop()
        switch (level) {
            case LogLevel.Error:
                return console.error(text)

            case LogLevel.Warning:
                return console.warn(text)

            case LogLevel.Info:
                return console.info(text)

            case LogLevel.Verbose:
                return console.debug(text)
        }
    }
}

export const msal = new PublicClientApplication(config)

export async function login(): Promise<void> {
    const request: PopupRequest = {
        redirectUri: config.auth.redirectUri,
        scopes: ["api://places.cluster.azure.ludimus.de/ReadWrite"],
    }
    return msal
        .initialize()
        .then(() => msal.acquireTokenSilent(request)
            .catch(() => msal.acquireTokenPopup(request)))
        .then(value => msal.setActiveAccount(value.account))
        .then(() => undefined);
}

export async function getToken(): Promise<string> {
    const request: PopupRequest = {
        redirectUri: config.auth.redirectUri,
        scopes: ["api://places.cluster.azure.ludimus.de/ReadWrite"],
    }
    return msal.acquireTokenSilent(request)
        .catch(() => msal.acquireTokenPopup(request))
        .then(value => {
            return value.accessToken;
        });
}