import {
  Hr,
  Html,
  Img,
  Text,
  Body,
  Head,
  Heading,
  Preview,
  Section,
  Container,
} from "@react-email/components";

interface OtpEmailProps {
  otp: string;
  appName: string;
  expiresInMinutes: string;
}

export function EmailOTP({ otp, appName, expiresInMinutes }: OtpEmailProps) {
  return (
    <Html>
      <Head />
      <Preview>{`${otp} is your ${appName} sign-in code`}</Preview>
      <Body style={main}>
        <Container style={container}>
          <Heading style={heading}>
            <Img
              src="https://desk0.xyz/favicon.ico"
              width={32}
              height={32}
              alt={appName}
            />
          </Heading>
          <Text style={paragraph}>
            Use the code below to sign in. This code expires in{" "}
            {expiresInMinutes} minutes.
          </Text>
          <Section style={codeBox}>
            <Text style={codeText}>{otp}</Text>
          </Section>
          <Text style={paragraph}>
            If you didn't request this code, you can safely ignore this email.
          </Text>
          <Hr style={hr} />
          <Text style={footer}>{appName}</Text>
        </Container>
      </Body>
    </Html>
  );
}

const main = {
  backgroundColor: "#f6f6f6",
  fontFamily: '-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif',
};

const container = {
  backgroundColor: "#ffffff",
  margin: "0 auto",
  padding: "32px 24px",
  maxWidth: "420px",
  borderRadius: "8px",
};

const heading = {
  fontSize: "20px",
  fontWeight: 700,
  color: "#111111",
  margin: "0 0 16px",
};

const paragraph = {
  fontSize: "14px",
  lineHeight: "22px",
  color: "#444444",
  margin: "0 0 16px",
};

const codeBox = {
  background: "#f2f2f2",
  borderRadius: "6px",
  padding: "16px",
  textAlign: "center" as const,
  margin: "0 0 24px",
};

const codeText = {
  fontSize: "32px",
  fontWeight: 700,
  letterSpacing: "8px",
  color: "#111111",
  margin: 0,
};

const hr = {
  borderColor: "#eeeeee",
  margin: "24px 0",
};

const footer = {
  fontSize: "12px",
  color: "#999999",
  margin: 0,
};
