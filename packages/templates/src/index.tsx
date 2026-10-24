import { format } from "util";
import { render } from "react-email";
import { mkdir, writeFile } from "fs/promises";

import { EmailOTP } from "./emails/email-otp";

const templates = [
  {
    name: "email-otp",
    component: (
      <EmailOTP
        otp="__OTP_CODE__"
        appName="__APP_NAME__"
        expiresInMinutes="__OTP_EXPIRES_IN_MINUTES__"
      />
    ),
  },
];

async function main() {
  await mkdir("dist", { recursive: true });

  for (const template of templates) {
    const html = await render(template.component);

    await writeFile(format("./dist/%s.html", template.name), html);
  }
}

main();
