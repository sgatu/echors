<?php

enum EchoRSCommands: string
{
    case Info = "\x00\x00";
    case Test = "\x01\x00";
    case SetString = "\x02\x00";
    case SetInt = "\x03\x00";
    case SetFloat = "\x04\x00";
    case Get = "\x05\x00";
    case Delete = "\x06\x00";
    case IncrementInt = "\x07\x00";
    case IncrementFloat = "\x08\x00";
    case ListPush = "\x09\x00";
    case ListPop = "\x0a\x00";
    case ListRange = "\x0b\x00";
    case ListExtract = "\x0c\x00";
    case ListLength = "\x0d\x00";
    case HLLAdd = "\x0e\x00";
    case HLLCount = "\x0f\x00";
    case HLLReset = "\x10\x00";
    case Unknown = "\x11\x00";
}
enum EchoRSCommandResult: string
{
    case OK = "\x01";
    case ERROR = "\x02";
}
enum ResultType: int
{
    case INT = 1;
    case FLOAT = 2;
    case STRING = 3;
    case LIST = 4;
    case MAP = 5;
    case LONG = 6;
}
class EchoRSClient
{

    /**
     * 
     * @var resource | null
     */
    private $fp = null;
    public function __construct(string $ip, int $port)
    {
        $this->fp = fsockopen("tcp://$ip", $port, $errno, $errstr);
        if (!$this->fp)
            throw new Exception("Could not connect to server");
    }
    private function processCommand(string $cmd)
    {
        $len = strlen($cmd);
        $lens = pack('V', $len);
        fwrite($this->fp, $lens . $cmd);
        fflush($this->fp);
        $response = fread($this->fp, 8096);
        $result = EchoRSCommandResult::from($response[0]);
        $response = substr($response, 1);
        return [
            "status" => $result->name,
            "result" => $this->interpretValue($response)
        ];
    }
    public function setString(string $key, string $value)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::SetString->value . $keylen . $key . pack('V', strlen($value)) . $value;
        return $this->processCommand($cmd);
    }
    public function incrementFloat(string $key, ?float $by = null)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::IncrementFloat->value . $keylen . $key;
        if ($by !== null) {
            $byB = pack('f', $by);
            $byBL = pack('V', strlen($byB));
            $cmd .= $byBL . $byB;
        }
        return $this->processCommand($cmd);
    }
    public function incrementInt(string $key, ?int $by = null)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::IncrementInt->value . $keylen . $key;
        if ($by !== null) {
            $byB = pack('V', $by);
            $byBL = pack('V', strlen($byB));
            $cmd .= $byBL . $byB;
        }
        return $this->processCommand($cmd);
    }
    public function get(string $key)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::Get->value . $keylen . $key;
        return $this->processCommand($cmd);
    }
    public function listPush(string $key, array $values)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::ListPush->value . $keylen . $key;
        foreach ($values as $val) {
            //force parse to string
            $strVal = strval($val);
            $len = pack('V', strlen($strVal));
            $cmd .= $len . $strVal;
        }
        return $this->processCommand($cmd);
    }
    public function listRange(string $key, int $start, int $end)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::ListRange->value . $keylen . $key;
        $startB = pack('V', $start);
        $startBL = pack('V', strlen($startB));
        $endB = pack('V', $end);
        $endBL = pack('V', strlen($endB));
        $cmd .= $startBL . $startB . $endBL . $endB;
        return $this->processCommand($cmd);
    }
    public function listExtract(string $key, int $start, int $end)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::ListExtract->value . $keylen . $key;
        $startB = pack('V', $start);
        $startBL = pack('V', strlen($startB));
        $endB = pack('V', $end);
        $endBL = pack('V', strlen($endB));
        $cmd .= $startBL . $startB . $endBL . $endB;
        return $this->processCommand($cmd);
    }
    public function listLength(string $key)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::ListLength->value . $keylen . $key;
        return $this->processCommand($cmd);
    }
    public function hllAdd(string $key, array $values)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::HLLAdd->value . $keylen . $key;
        foreach ($values as $val) {
            //force parse to string
            $strVal = strval($val);
            $len = pack('V', strlen($strVal));
            $cmd .= $len . $strVal;
        }
        return $this->processCommand($cmd);
    }
    public function hllCount(string $key)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::HLLCount->value . $keylen . $key;
        return $this->processCommand($cmd);
    }
    public function hllReset(string $key)
    {
        $keylen = pack('V', strlen($key));
        $cmd = EchoRSCommands::HLLReset->value . $keylen . $key;
        return $this->processCommand($cmd);
    }
    private function interpretList(string $data)
    {
        $data = substr($data, 1); // remove byte of list type
        $lst = [];
        while (strlen($data) > 0) {
            $lst[] = $this->interpretString($data);
        }
        return $lst;
    }
    private function interpretString(string &$data)
    {
        $data = substr($data, 1); //remove byte of string type
        $len = unpack('V', substr($data, 0, 4))[1];
        $str = substr($data, 4, min($len, strlen($data) - 4));
        $data = substr($data, $len + 4);
        return $str;
    }
    private function interpretValue(string $data)
    {
        $type = ResultType::from(ord($data[0]));
        return match ($type) {
            ResultType::INT => unpack('V', substr($data, 1))[1],
            ResultType::FLOAT => unpack('f', substr($data, 1))[1],
            ResultType::STRING => $this->interpretString($data),
            ResultType::LIST => $this->interpretList($data),
            ResultType::LONG => unpack('P', substr($data, 1))[1]
        };
    }
}
