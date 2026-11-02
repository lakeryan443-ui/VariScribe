!macro NSIS_HOOK_POSTINSTALL
  ${GetTime} "" "L" $0 $1 $2 $3 $4 $5 $6
  WriteRegStr SHCTX "${UNINSTKEY}" "AgreementVersion" "1.0"
  WriteRegStr SHCTX "${UNINSTKEY}" "AcceptedAtLocal" "$2-$1-$0 $4:$5:$6"
!macroend
